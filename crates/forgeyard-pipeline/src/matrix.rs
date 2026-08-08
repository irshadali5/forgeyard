use itertools::Itertools;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct MatrixContext {
    pub variables: BTreeMap<String, String>,
}

impl MatrixContext {
    pub fn substitute(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (k, v) in &self.variables {
            result = result.replace(&format!("${{{}}}", k), v);
            result = result.replace(&format!("${}", k), v);
        }
        result
    }

    pub fn substitute_command(&self, command: &[String]) -> Vec<String> {
        command.iter().map(|arg| self.substitute(arg)).collect()
    }
}

pub struct MatrixExpander;

impl MatrixExpander {
    /// Takes a sequence of strings (e.g. "os: [linux, windows]") and expands them into a Cartesian product using `itertools`.
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
                    let mut values: Vec<String> = parts[1]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if values.is_empty() {
                        values.push("default".to_string());
                    }
                    dimensions.push((key, values));
                }
            }

            if dimensions.is_empty() {
                return vec![MatrixContext { variables: BTreeMap::new() }];
            }

            let value_iters: Vec<Vec<(String, String)>> = dimensions
                .into_iter()
                .map(|(key, vals)| vals.into_iter().map(|v| (key.clone(), v)).collect())
                .collect();

            value_iters
                .into_iter()
                .multi_cartesian_product()
                .map(|tuple| {
                    let mut variables = BTreeMap::new();
                    for (k, v) in tuple {
                        variables.insert(k, v);
                    }
                    MatrixContext { variables }
                })
                .collect()
        } else {
            vec![MatrixContext { variables: BTreeMap::new() }]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_expansion_and_substitution() {
        let matrix_def = vec![
            "os: linux, windows".to_string(),
            "arch: x86_64, aarch64".to_string(),
        ];

        let contexts = MatrixExpander::expand(&Some(matrix_def));
        assert_eq!(contexts.len(), 4);

        let first = &contexts[0];
        let subbed = first.substitute("cargo build --target ${arch}-${os}");
        assert!(subbed.contains("x86_64") || subbed.contains("aarch64"));
    }
}
