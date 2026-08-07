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

    pub fn generate_token_efficient_summary(result: &ExtractionResult) -> String {
        let mut summary = String::new();
        summary.push_str("### Codebase Knowledge Graph Summary\n");
        summary.push_str(&format!("- Total Entities/Nodes: {}\n", result.nodes.len()));
        summary.push_str(&format!("- Total Dependencies/Edges: {}\n", result.edges.len()));
        summary.push_str("\n#### Primary Entities & Modules:\n");
        
        for node in result.nodes.iter().take(25) {
            summary.push_str(&format!("- [`{}`] ({:?}) in `{}`\n", node.label, node.node_type, node.source_file));
        }
        
        if result.nodes.len() > 25 {
            summary.push_str(&format!("... and {} more entities.\n", result.nodes.len() - 25));
        }

        summary
    }
}
