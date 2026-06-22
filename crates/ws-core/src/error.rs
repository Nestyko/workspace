use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum WorkspaceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML serialization/deserialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Catalog error: {0}")]
    Catalog(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Provider error ({provider}): {message}")]
    Provider {
        provider: String,
        message: String,
    },

    #[error("Workspace creation failed: {0}")]
    Workspace(String),

    #[error("Editor error ({editor}): {message}")]
    Editor {
        editor: String,
        message: String,
    },

    #[error("Command execution error: {0}")]
    Command(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl WorkspaceError {
    pub fn provider(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
        }
    }

    pub fn editor(editor: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Editor {
            editor: editor.into(),
            message: message.into(),
        }
    }
}
