use crate::editors::EditorAdapter;
use crate::models::LocalConfig;
use crate::providers::{CodeProvider, IssueProvider, DocProvider};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct CommandContext {
    pub config: LocalConfig,
    pub workspace_root: PathBuf,
    pub issue_provider: Option<Arc<dyn IssueProvider>>,
    pub code_provider: Option<Arc<dyn CodeProvider>>,
    pub doc_provider: Option<Arc<dyn DocProvider>>,
    pub editor_adapters: HashMap<String, Arc<dyn EditorAdapter>>,
}

impl CommandContext {
    pub fn new(
        config: LocalConfig,
        workspace_root: PathBuf,
        issue_provider: Option<Arc<dyn IssueProvider>>,
        code_provider: Option<Arc<dyn CodeProvider>>,
        doc_provider: Option<Arc<dyn DocProvider>>,
        editor_adapters: HashMap<String, Arc<dyn EditorAdapter>>,
    ) -> Self {
        Self {
            config,
            workspace_root,
            issue_provider,
            code_provider,
            doc_provider,
            editor_adapters,
        }
    }
}
