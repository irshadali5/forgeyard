use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::fs::File;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

fn ensure_parent_dir(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

pub fn create_tar_gz(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), ArchiveError> {
    let dst_path = dst.as_ref();
    ensure_parent_dir(dst_path)?;

    let tar_gz = File::create(dst_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(".", src)?;
    tar.finish()?;
    Ok(())
}

pub fn unpack_tar_gz(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), ArchiveError> {
    let dst_path = dst.as_ref();
    ensure_parent_dir(dst_path)?;

    let tar_gz = File::open(src)?;
    let dec = GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(dec);
    archive.unpack(dst_path)?;
    Ok(())
}

pub fn create_zip(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), ArchiveError> {
    let dst_path = dst.as_ref();
    ensure_parent_dir(dst_path)?;

    let file = File::create(dst_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let src_path = src.as_ref();
    if src_path.is_dir() {
        for entry in walkdir::WalkDir::new(src_path) {
            let entry = entry.map_err(std::io::Error::other)?;
            let path = entry.path();
            let name = path.strip_prefix(src_path).map_err(std::io::Error::other)?;

            if path.is_file() {
                zip.start_file(name.to_string_lossy(), options)?;
                let mut f = File::open(path)?;
                std::io::copy(&mut f, &mut zip)?;
            } else if !name.as_os_str().is_empty() {
                zip.add_directory(name.to_string_lossy(), options)?;
            }
        }
    } else {
        let file_name = src_path.file_name().ok_or_else(|| std::io::Error::other("Invalid file name"))?;
        zip.start_file(file_name.to_string_lossy(), options)?;
        let mut f = File::open(src_path)?;
        std::io::copy(&mut f, &mut zip)?;
    }

    zip.finish()?;
    Ok(())
}

pub fn unpack_zip(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), ArchiveError> {
    let dst_path = dst.as_ref();
    ensure_parent_dir(dst_path)?;

    let file = File::open(src)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(dst_path)?;
    Ok(())
}

pub enum ArchiveFormat {
    TarGz,
    Zip,
    Unknown,
}

pub struct ArchiveManager;

impl ArchiveManager {
    pub fn detect_format(path: impl AsRef<Path>) -> ArchiveFormat {
        let p = path.as_ref();
        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
            if ext == "zip" {
                return ArchiveFormat::Zip;
            } else if ext == "gz" || ext == "tgz" {
                return ArchiveFormat::TarGz;
            }
        }
        ArchiveFormat::Unknown
    }

    pub fn auto_extract(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), ArchiveError> {
        match Self::detect_format(&src) {
            ArchiveFormat::TarGz => unpack_tar_gz(src, dst),
            ArchiveFormat::Zip => unpack_zip(src, dst),
            ArchiveFormat::Unknown => unpack_tar_gz(src, dst),
        }
    }

    pub fn calculate_compression_ratio(src_dir: impl AsRef<Path>, archive_file: impl AsRef<Path>) -> Result<f64, ArchiveError> {
        let src_size = walkdir::WalkDir::new(src_dir.as_ref())
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|e| e.metadata().ok())
            .filter(|m| m.is_file())
            .map(|m| m.len())
            .sum::<u64>() as f64;

        let archive_size = std::fs::metadata(archive_file.as_ref())?.len() as f64;
        if src_size == 0.0 {
            return Ok(1.0);
        }
        Ok(archive_size / src_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_tar_gz_creation_and_unpacking() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("test.txt"), b"archive payload content").unwrap();

        let archive_file = dir.path().join("test.tar.gz");
        create_tar_gz(&src_dir, &archive_file).unwrap();
        assert!(archive_file.exists());

        let unpack_dir = dir.path().join("unpack");
        unpack_tar_gz(&archive_file, &unpack_dir).unwrap();
        assert!(unpack_dir.join("test.txt").exists());
    }

    #[test]
    fn test_zip_creation_and_unpacking() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src_zip");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("hello.txt"), b"zip payload content").unwrap();

        let zip_file = dir.path().join("test.zip");
        create_zip(&src_dir, &zip_file).unwrap();
        assert!(zip_file.exists());

        let unpack_dir = dir.path().join("unpack_zip");
        unpack_zip(&zip_file, &unpack_dir).unwrap();
        assert!(unpack_dir.join("hello.txt").exists());
    }
}
