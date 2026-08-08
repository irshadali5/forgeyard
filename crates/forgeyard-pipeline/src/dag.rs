use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};

#[derive(Debug, thiserror::Error)]
pub enum DagError {
    #[error("Cycle detected in pipeline involving job: {0}")]
    CycleDetected(String),
    #[error("Job {0} depends on missing job {1}")]
    MissingDependency(String, String),
}

pub struct PipelineDag {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
    edges_raw: Vec<(String, String)>,
}

impl Default for PipelineDag {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineDag {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            edges_raw: Vec::new(),
        }
    }

    pub fn add_node(&mut self, job: String) {
        if !self.node_map.contains_key(&job) {
            let idx = self.graph.add_node(job.clone());
            self.node_map.insert(job, idx);
        }
    }

    pub fn add_edge(&mut self, from: String, to: String) {
        self.add_node(from.clone());
        self.add_node(to.clone());
        self.edges_raw.push((from, to));
    }

    /// Performs a topological sort using `petgraph` and returns execution order, validating cycles and missing deps.
    pub fn validate(&self, known_jobs: &HashSet<String>) -> Result<Vec<String>, DagError> {
        let mut graph = DiGraph::<String, ()>::new();
        let mut node_map = HashMap::new();

        for node in self.node_map.keys() {
            if !known_jobs.contains(node) {
                let dependent = self
                    .edges_raw
                    .iter()
                    .find(|(_, target)| target == node)
                    .map(|(src, _)| src.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                return Err(DagError::MissingDependency(dependent, node.clone()));
            }
            let idx = graph.add_node(node.clone());
            node_map.insert(node.clone(), idx);
        }

        for (from, to) in &self.edges_raw {
            if let (Some(&from_idx), Some(&to_idx)) = (node_map.get(from), node_map.get(to)) {
                graph.add_edge(from_idx, to_idx, ());
            }
        }

        match toposort(&graph, None) {
            Ok(order) => {
                let sorted = order.into_iter().map(|idx| graph[idx].clone()).collect();
                Ok(sorted)
            }
            Err(cycle) => {
                let cycle_node = graph[cycle.node_id()].clone();
                Err(DagError::CycleDetected(cycle_node))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_dag_execution_order() {
        let mut dag = PipelineDag::new();
        dag.add_node("build".to_string());
        dag.add_node("test".to_string());
        dag.add_edge("build".to_string(), "test".to_string());

        let mut known = HashSet::new();
        known.insert("build".to_string());
        known.insert("test".to_string());

        let order = dag.validate(&known).expect("DAG validation failed");
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], "build");
        assert_eq!(order[1], "test");
    }

    #[test]
    fn test_cycle_detection() {
        let mut dag = PipelineDag::new();
        dag.add_edge("A".to_string(), "B".to_string());
        dag.add_edge("B".to_string(), "A".to_string());

        let mut known = HashSet::new();
        known.insert("A".to_string());
        known.insert("B".to_string());

        let result = dag.validate(&known);
        assert!(result.is_err());
        match result {
            Err(DagError::CycleDetected(_)) => {}
            _ => panic!("Expected CycleDetected error"),
        }
    }
}
