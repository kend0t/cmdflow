use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use std::fs;
use std::path::PathBuf;

/// A single workflow consisting of a name, description, and ordered list of commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub commands: Vec<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

/// The top-level store that holds all workflows.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStore {
    pub workflows: Vec<Workflow>,
}

impl WorkflowStore {
    /// Returns the path to the workflows JSON file.
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cmdflow");
        config_dir.join("workflows.json")
    }

    /// Load workflows from disk. Creates an empty store if file doesn't exist.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
            serde_json::from_str(&data).unwrap_or(WorkflowStore {
                workflows: Vec::new(),
            })
        } else {
            WorkflowStore {
                workflows: Vec::new(),
            }
        }
    }

    /// Save workflows to disk. Auto-creates directory if needed.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)?;
        Ok(())
    }

    /// Add a new workflow.
    pub fn add(&mut self, workflow: Workflow) {
        self.workflows.push(workflow);
    }

    /// Get a workflow by name.
    pub fn get(&self, name: &str) -> Option<&Workflow> {
        self.workflows.iter().find(|w| w.name == name)
    }

    /// Get a mutable workflow by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Workflow> {
        self.workflows.iter_mut().find(|w| w.name == name)
    }

    /// Delete a workflow by name. Returns true if found and deleted.
    pub fn delete(&mut self, name: &str) -> bool {
        let len_before = self.workflows.len();
        self.workflows.retain(|w| w.name != name);
        self.workflows.len() < len_before
    }

    /// List all workflow names.
    pub fn names(&self) -> Vec<&str> {
        self.workflows.iter().map(|w| w.name.as_str()).collect()
    }
}

impl Workflow {
    /// Create a new workflow with the current timestamp.
    pub fn new(name: String, description: String, commands: Vec<String>) -> Self {
        let now = Local::now();
        Workflow {
            name,
            description,
            commands,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        let wf = Workflow::new(
            "test".to_string(),
            "A test workflow".to_string(),
            vec!["echo hello".to_string()],
        );
        assert_eq!(wf.name, "test");
        assert_eq!(wf.commands.len(), 1);
    }

    #[test]
    fn test_store_crud() {
        let mut store = WorkflowStore { workflows: Vec::new() };
        let wf = Workflow::new(
            "deploy".to_string(),
            "Deploy app".to_string(),
            vec!["echo deploying".to_string()],
        );
        store.add(wf);
        assert_eq!(store.names().len(), 1);
        assert!(store.get("deploy").is_some());
        assert!(store.delete("deploy"));
        assert!(store.get("deploy").is_none());
    }
}
