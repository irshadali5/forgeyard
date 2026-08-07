use forgeyard_model::JobId;
use std::collections::{HashMap, HashSet};

#[derive(Debug, thiserror::Error)]
pub enum DagError {
    #[error("Cycle detected in pipeline involving job: {0}")]
    CycleDetected(String),
    #[error("Job {0} depends on missing job {1}")]
    MissingDependency(String, String),
}

pub struct PipelineDag {
    edges: HashMap<String, Vec<String>>,
    nodes: HashSet<String>,
}

impl PipelineDag {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            nodes: HashSet::new(),
        }
    }

    pub fn add_node(&mut self, job: String) {
        self.nodes.insert(job.clone());
        self.edges.entry(job).or_insert_with(Vec::new);
    }

    pub fn add_edge(&mut self, from: String, to: String) {
        self.nodes.insert(from.clone());
        self.nodes.insert(to.clone());
        self.edges.entry(from).or_insert_with(Vec::new).push(to);
    }

    /// Performs a topological sort and returns the execution order, validating cycles and missing deps.
    pub fn validate(&self, known_jobs: &HashSet<String>) -> Result<Vec<String>, DagError> {
        let mut in_degree = HashMap::new();
        for node in &self.nodes {
            if !known_jobs.contains(node) {
                // Determine who depends on this missing node for error context
                let dependent = self.edges.iter()
                    .find(|(_, targets)| targets.contains(node))
                    .map(|(src, _)| src.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                return Err(DagError::MissingDependency(dependent, node.clone()));
            }
            in_degree.insert(node.clone(), 0);
        }

        for targets in self.edges.values() {
            for target in targets {
                *in_degree.entry(target.clone()).or_insert(0) += 1;
            }
        }

        let mut queue = Vec::new();
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push(node.clone());
            }
        }

        let mut sorted = Vec::new();
        while let Some(node) = queue.pop() {
            sorted.push(node.clone());
            if let Some(targets) = self.edges.get(&node) {
                for target in targets {
                    let degree = in_degree.get_mut(target).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(target.clone());
                    }
                }
            }
        }

        if sorted.len() != self.nodes.len() {
            // Find a node that has non-zero in-degree
            let cycle_node = in_degree.into_iter()
                .find(|(_, degree)| *degree > 0)
                .map(|(node, _)| node)
                .unwrap_or_else(|| "unknown".to_string());
            return Err(DagError::CycleDetected(cycle_node));
        }

        Ok(sorted)
    }
}
