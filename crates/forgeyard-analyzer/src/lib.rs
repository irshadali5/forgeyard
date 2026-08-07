pub mod graph {
    use std::path::PathBuf;
    use graphify_extract::{collect_files, extract};
    use graphify_core::model::ExtractionResult;

    pub async fn extract_knowledge_graph(workspace_root: PathBuf) -> Result<ExtractionResult, String> {
        let files = tokio::task::spawn_blocking(move || {
            collect_files(&workspace_root)
        }).await.map_err(|e| format!("Failed to collect files: {}", e))?;

        let results = tokio::task::spawn_blocking(move || {
            extract(&files)
        }).await.map_err(|e| format!("Failed to extract graph: {}", e))?;

        Ok(results)
    }
}
