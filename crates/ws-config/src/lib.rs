use std::fs;
use std::path::{Path, PathBuf};
use ws_core::error::WorkspaceError;
use ws_core::models::LocalConfig;

pub fn find_workspace_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        // If we see Cargo.toml at the root, or .ws directory, we assume it's the root.
        if current.join("Cargo.toml").exists() || current.join(".ws").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    // Default fallback to current directory
    std::env::current_dir().ok()
}

pub fn get_config_path(root: &Path) -> PathBuf {
    root.join(".ws").join("config.yaml")
}

pub fn load_config(root: &Path) -> Result<LocalConfig, WorkspaceError> {
    let path = get_config_path(root);
    if !path.exists() {
        return Ok(LocalConfig::default());
    }
    let content = fs::read_to_string(&path)?;
    let config: LocalConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn save_config(root: &Path, config: &LocalConfig) -> Result<(), WorkspaceError> {
    let path = get_config_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_yaml::to_string(config)?;
    fs::write(&path, content)?;
    Ok(())
}
