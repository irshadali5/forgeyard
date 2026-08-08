pub mod graph {
    use std::path::PathBuf;
    use graphify_extract::{collect_files, extract};
    use graphify_core::model::ExtractionResult;
    use forgeyard_model::{CodeGraph, SymbolInfo, SymbolKind, CallEdge};

    pub async fn extract_knowledge_graph(workspace_root: PathBuf) -> Result<ExtractionResult, String> {
        let files = tokio::task::spawn_blocking(move || {
            collect_files(&workspace_root)
        }).await.map_err(|e| format!("Failed to collect files: {}", e))?;

        let results = tokio::task::spawn_blocking(move || {
            extract(&files)
        }).await.map_err(|e| format!("Failed to extract graph: {}", e))?;

        Ok(results)
    }

    /// Converts raw graphify extraction results into an enriched CodeGraph model.
    pub fn build_code_graph(result: &ExtractionResult) -> CodeGraph {
        let mut symbols = Vec::new();
        let mut edges = Vec::new();

        for (idx, node) in result.nodes.iter().enumerate() {
            let kind = match format!("{:?}", node.node_type).to_lowercase().as_str() {
                s if s.contains("function") => SymbolKind::Function,
                s if s.contains("method") => SymbolKind::Method,
                s if s.contains("struct") => SymbolKind::Struct,
                s if s.contains("enum") => SymbolKind::Enum,
                s if s.contains("trait") => SymbolKind::Trait,
                s if s.contains("module") => SymbolKind::Module,
                s if s.contains("interface") => SymbolKind::Interface,
                s if s.contains("class") => SymbolKind::Class,
                _ => SymbolKind::Variable,
            };

            let is_public = node.label.starts_with("pub") || node.label.contains("export ");

            symbols.push(SymbolInfo {
                symbol_id: format!("sym-{}", idx),
                label: node.label.clone(),
                kind,
                file_path: node.source_file.clone(),
                line: 1,
                signature: Some(node.label.clone()),
                is_public,
            });
        }

        for edge in result.edges.iter() {
            edges.push(CallEdge {
                caller_id: edge.source.clone(),
                callee_id: edge.target.clone(),
                line_number: 1,
            });
        }

        let total_nodes = symbols.len();
        let total_edges = edges.len();

        CodeGraph {
            symbols,
            edges,
            total_nodes,
            total_edges,
        }
    }

    /// RTK (Real-Time Knowledge / Token Compression Filter)
    pub struct RtkCompressor {
        pub max_token_budget: usize,
        pub preserve_signatures: bool,
    }

    impl RtkCompressor {
        pub fn new(max_token_budget: usize) -> Self {
            Self {
                max_token_budget,
                preserve_signatures: true,
            }
        }

        /// Compresses a CodeGraph into a token-optimized markdown representation
        pub fn compress(&self, graph: &CodeGraph) -> String {
            let mut output = String::new();
            output.push_str("### RTK-Compressed Codebase Knowledge Graph\n");
            output.push_str(&format!("- Total Symbols: {} | Call Edges: {}\n", graph.total_nodes, graph.total_edges));
            output.push_str("#### Public API Surface & Core Entities:\n");

            let mut count = 0;
            // Prioritize public symbols first, then others
            let mut sorted_symbols = graph.symbols.clone();
            sorted_symbols.sort_by_key(|s| !s.is_public);

            for sym in &sorted_symbols {
                if output.len() >= self.max_token_budget {
                    output.push_str(&format!("\n... [RTK Truncated: {} symbols omitted for token budget]\n", graph.symbols.len() - count));
                    break;
                }

                let pub_prefix = if sym.is_public { "pub " } else { "" };
                output.push_str(&format!(
                    "- [{}{:?}] `{}` in `{}`\n",
                    pub_prefix, sym.kind, sym.label, sym.file_path
                ));
                count += 1;
            }

            output
        }
    }

    pub fn generate_token_efficient_summary(result: &ExtractionResult) -> String {
        let code_graph = build_code_graph(result);
        let compressor = RtkCompressor::new(4000);
        compressor.compress(&code_graph)
    }

    pub struct AiPatchGenerator;

    impl AiPatchGenerator {
        pub fn propose_patch(
            failed_test_name: &str,
            error_trace: &str,
            target_file: &str,
            rtk_summary: Option<&str>,
        ) -> String {
            let mut patch = String::new();
            patch.push_str(&format!("--- a/{}\n", target_file));
            patch.push_str(&format!("+++ b/{}\n", target_file));
            patch.push_str("@@ -1,5 +1,6 @@\n");
            patch.push_str(&format!("// Autonomous AI Remediation Patch for: {}\n", failed_test_name));
            patch.push_str(&format!("// Root Cause Trace: {}\n", error_trace.lines().next().unwrap_or("Assertion failed")));
            if let Some(summary) = rtk_summary {
                let context_line = summary.lines().find(|l| l.contains("Public API Surface")).unwrap_or("// RTK AST Context attached");
                patch.push_str(&format!("// AST Context: {}\n", context_line));
            }
            patch
        }
    }
}

#[cfg(test)]
mod tests {
    use super::graph::*;
    use forgeyard_model::{CodeGraph, SymbolInfo, SymbolKind};

    #[test]
    fn test_rtk_compressor_truncation() {
        let mut symbols = Vec::new();
        for i in 0..100 {
            symbols.push(SymbolInfo {
                symbol_id: format!("sym-{}", i),
                label: format!("pub fn test_function_{}()", i),
                kind: SymbolKind::Function,
                file_path: "src/lib.rs".to_string(),
                line: i as u32,
                signature: Some(format!("fn test_function_{}()", i)),
                is_public: i % 2 == 0,
            });
        }

        let graph = CodeGraph {
            symbols,
            edges: vec![],
            total_nodes: 100,
            total_edges: 0,
        };

        let compressor = RtkCompressor::new(500);
        let summary = compressor.compress(&graph);

        assert!(summary.contains("RTK-Compressed Codebase Knowledge Graph"));
        assert!(summary.contains("RTK Truncated"));
    }

    #[test]
    fn test_ai_patch_generator() {
        let patch = AiPatchGenerator::propose_patch("test_foo", "assertion failed: x == y", "src/foo.rs", Some("#### Public API Surface"));
        assert!(patch.contains("Autonomous AI Remediation Patch"));
        assert!(patch.contains("--- a/src/foo.rs"));
        assert!(patch.contains("AST Context"));
    }
}
