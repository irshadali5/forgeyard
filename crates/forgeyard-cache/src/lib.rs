use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache error: {0}")]
    Failed(String),
}

pub struct CacheKey(pub String);

#[async_trait]
pub trait Cache: Send + Sync {
    async fn put(&self, key: &CacheKey, data: &[u8]) -> Result<(), CacheError>;
    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>, CacheError>;
}

pub struct LocalDirectoryCache {
    pub dir: std::path::PathBuf,
}

impl LocalDirectoryCache {
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Self {
        Self { dir: dir.into() }
    }
}

#[async_trait]
impl Cache for LocalDirectoryCache {
    async fn put(&self, key: &CacheKey, data: &[u8]) -> Result<(), CacheError> {
        let path = self.dir.join(&key.0);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CacheError::Failed(e.to_string()))?;
        }
        std::fs::write(path, data).map_err(|e| CacheError::Failed(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>, CacheError> {
        let path = self.dir.join(&key.0);
        if !path.exists() {
            return Ok(None);
        }
        match std::fs::read(path) {
            Ok(data) => Ok(Some(data)),
            Err(e) => Err(CacheError::Failed(e.to_string())),
        }
    }
}

pub struct MemoryCache {
    cache: quick_cache::sync::Cache<String, Vec<u8>>,
}

impl MemoryCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: quick_cache::sync::Cache::new(capacity),
        }
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn put(&self, key: &CacheKey, data: &[u8]) -> Result<(), CacheError> {
        self.cache.insert(key.0.clone(), data.to_vec());
        Ok(())
    }

    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>, CacheError> {
        Ok(self.cache.get(&key.0))
    }
}

pub struct DiskCache {
    db: redb::Database,
}

const CACHE_TABLE: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new("cache");

impl DiskCache {
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Result<Self, CacheError> {
        let path = dir.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CacheError::Failed(e.to_string()))?;
        }
        let db_path = path.join("cache.redb");
        let db = if db_path.exists() {
            redb::Database::open(&db_path).map_err(|e| CacheError::Failed(e.to_string()))?
        } else {
            redb::Database::create(&db_path).map_err(|e| CacheError::Failed(e.to_string()))?
        };
            
        let write_txn = db.begin_write().map_err(|e| CacheError::Failed(e.to_string()))?;
        write_txn.open_table(CACHE_TABLE).map_err(|e| CacheError::Failed(e.to_string()))?;
        write_txn.commit().map_err(|e| CacheError::Failed(e.to_string()))?;
            
        Ok(Self { db })
    }
}

#[async_trait]
impl Cache for DiskCache {
    async fn put(&self, key: &CacheKey, data: &[u8]) -> Result<(), CacheError> {
        let write_txn = self.db.begin_write().map_err(|e| CacheError::Failed(e.to_string()))?;
        {
            let mut table = write_txn.open_table(CACHE_TABLE).map_err(|e| CacheError::Failed(e.to_string()))?;
            table.insert(key.0.as_str(), data).map_err(|e| CacheError::Failed(e.to_string()))?;
        }
        write_txn.commit().map_err(|e| CacheError::Failed(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>, CacheError> {
        let read_txn = self.db.begin_read().map_err(|e| CacheError::Failed(e.to_string()))?;
        let table = match read_txn.open_table(CACHE_TABLE) {
            Ok(t) => t,
            Err(_) => return Ok(None),
        };
        let item = table.get(key.0.as_str()).map_err(|e| CacheError::Failed(e.to_string()))?;
        
        Ok(item.map(|i| i.value().to_vec()))
    }
}

pub struct TieredCache {
    l1_memory: MemoryCache,
    l2_disk: DiskCache,
}

impl TieredCache {
    pub fn new(memory_capacity: usize, disk_dir: impl Into<std::path::PathBuf>) -> Result<Self, CacheError> {
        let l1_memory = MemoryCache::new(memory_capacity);
        let l2_disk = DiskCache::new(disk_dir)?;
        Ok(Self { l1_memory, l2_disk })
    }
}

#[async_trait]
impl Cache for TieredCache {
    async fn put(&self, key: &CacheKey, data: &[u8]) -> Result<(), CacheError> {
        self.l1_memory.put(key, data).await?;
        self.l2_disk.put(key, data).await?;
        Ok(())
    }

    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>, CacheError> {
        // Step 1: Check L1 memory cache
        if let Ok(Some(data)) = self.l1_memory.get(key).await {
            return Ok(Some(data));
        }

        // Step 2: Fallback to L2 disk cache
        if let Ok(Some(data)) = self.l2_disk.get(key).await {
            // Populate L1 memory cache on L2 hit
            let _ = self.l1_memory.put(key, &data).await;
            return Ok(Some(data));
        }

        Ok(None)
    }
}
