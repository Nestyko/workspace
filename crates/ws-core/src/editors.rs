use crate::error::WorkspaceError;
use crate::models::{OpenServiceInput, OpenWorkspaceInput};
use async_trait::async_trait;

#[async_trait]
pub trait EditorAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;

    async fn is_available(&self) -> Result<bool, WorkspaceError>;

    async fn open_workspace(&self, input: OpenWorkspaceInput) -> Result<(), WorkspaceError>;

    async fn open_service(&self, input: OpenServiceInput) -> Result<(), WorkspaceError>;
}
