pub mod command;
pub mod context;
pub mod editors;
pub mod error;
pub mod models;
pub mod providers;

// Re-export common traits and structs
pub use command::{AiCommand, CommandRegistry, ErasedAiCommand};
pub use context::CommandContext;
pub use editors::EditorAdapter;
pub use error::WorkspaceError;
pub use models::*;
pub use providers::{CodeProvider, DocProvider, IssueProvider};
