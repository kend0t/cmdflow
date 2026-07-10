use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Reads shell command history and returns deduplicated commands (most recent first).
/// Supports PowerShell (Windows), zsh, bash, and fish (macOS/Linux).
pub fn read_shell_history() -> Vec<String> {
    let history_paths = get_history_paths();

    for path in history_paths {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                let mut seen = HashSet::new();
                let mut result = Vec::new();

                for line in content.lines().rev() {
                    // zsh extended history format: ": timestamp:0;command"
                    let cleaned = if line.starts_with(": ") {
                        line.splitn(2, ';').nth(1).unwrap_or(line)
                    } else {
                        line
                    };
                    let trimmed = cleaned.trim();
                    if !trimmed.is_empty() && seen.insert(trimmed.to_string()) {
                        result.push(trimmed.to_string());
                    }
                }

                if !result.is_empty() {
                    return result;
                }
            }
        }
    }

    Vec::new()
}

/// Returns possible shell history file paths for the current OS.
fn get_history_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Windows: PowerShell PSReadline history
    if cfg!(target_os = "windows") {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            paths.push(
                PathBuf::from(appdata)
                    .join("Microsoft")
                    .join("Windows")
                    .join("PowerShell")
                    .join("PSReadline")
                    .join("ConsoleHost_history.txt"),
            );
        }
    }

    // macOS / Linux: zsh, bash, fish
    if let Some(home) = dirs::home_dir() {
        // zsh (default on macOS)
        paths.push(home.join(".zsh_history"));
        // bash
        paths.push(home.join(".bash_history"));
        // fish
        paths.push(
            home.join(".local")
                .join("share")
                .join("fish")
                .join("fish_history"),
        );
    }

    paths
}

/// Filters history by a search query (case-insensitive substring match).
pub fn filter_history(history: &[String], query: &str) -> Vec<String> {
    if query.is_empty() {
        return history.to_vec();
    }
    let query_lower = query.to_lowercase();
    history
        .iter()
        .filter(|cmd| cmd.to_lowercase().contains(&query_lower))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_history() {
        let history = vec![
            "gcloud config set project dev".to_string(),
            "cargo build".to_string(),
            "gcloud run deploy".to_string(),
            "echo hello".to_string(),
        ];
        let filtered = filter_history(&history, "gcloud");
        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].contains("gcloud"));
    }

    #[test]
    fn test_filter_empty_query() {
        let history = vec!["a".to_string(), "b".to_string()];
        let filtered = filter_history(&history, "");
        assert_eq!(filtered.len(), 2);
    }
}
