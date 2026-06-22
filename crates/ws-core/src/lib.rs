pub mod error;
pub mod models;
pub mod providers;
pub mod editors;
pub mod context;
pub mod command;

// Re-export common traits and structs
pub use error::WorkspaceError;
pub use models::*;
pub use providers::{CodeProvider, IssueProvider, DocProvider};
pub use editors::EditorAdapter;
pub use context::CommandContext;
pub use command::{AiCommand, ErasedAiCommand, CommandRegistry};
