use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct MatrixContext {
    pub variables: BTreeMap<String, String>,
}

pub struct MatrixExpander;

impl MatrixExpander {
    /// Takes a sequence of strings (e.g. "os: [linux, windows]") and expands them into a Cartesian product of variable maps.
    /// Example `matrix_def`: ["os: linux, windows", "arch: x86_64, aarch64"]
    /// In a production system, this would parse yaml/json arrays, but we will adapt to the current `ForgeyardConfig` which holds `Vec<String>`.
    pub fn expand(matrix_def: &Option<Vec<String>>) -> Vec<MatrixContext> {
        if let Some(def) = matrix_def {
            if def.is_empty() {
                return vec![MatrixContext { variables: BTreeMap::new() }];
            }

            let mut dimensions: Vec<(String, Vec<String>)> = Vec::new();

            for line in def {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim().to_string();
                    let values: Vec<String> = parts[1]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                    dimensions.push((key, values));
                }
            }

            let mut results = vec![BTreeMap::new()];

            for (key, values) in dimensions {
                let mut new_results = Vec::new();
                for current in &results {
                    for val in &values {
                        let mut next = current.clone();
                        next.insert(key.clone(), val.clone());
                        new_results.push(next);
                    }
                }
                results = new_results;
            }

            results.into_iter().map(|variables| MatrixContext { variables }).collect()
        } else {
            vec![MatrixContext { variables: BTreeMap::new() }]
        }
    }
}
