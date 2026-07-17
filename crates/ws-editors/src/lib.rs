use async_trait::async_trait;
use duct::cmd;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;
use ws_core::command::AiCommand;
use ws_core::context::CommandContext;
use ws_core::editors::EditorAdapter;
use ws_core::error::WorkspaceError;
use ws_core::models::{OpenServiceInput, OpenWorkspaceInput};

pub struct CursorEditorAdapter;

#[async_trait]
impl EditorAdapter for CursorEditorAdapter {
    fn id(&self) -> &'static str {
        "cursor"
    }

    fn label(&self) -> &'static str {
        "Cursor"
    }

    async fn is_available(&self) -> Result<bool, WorkspaceError> {
        let result = cmd("cursor", &["--version"])
            .stdout_null()
            .stderr_null()
            .run();
        Ok(result.is_ok())
    }

    async fn open_workspace(&self, input: OpenWorkspaceInput) -> Result<(), WorkspaceError> {
        let path = Path::new("workspaces")
            .join(&input.epic_key)
            .join(format!("{}.code-workspace", input.epic_key));
        info!("Opening workspace with Cursor: {}", path.display());
        cmd("cursor", &[path.to_string_lossy().into_owned()])
            .run()
            .map_err(|e| {
                WorkspaceError::editor("cursor", format!("Failed to open workspace: {}", e))
            })?;
        Ok(())
    }

    async fn open_service(&self, input: OpenServiceInput) -> Result<(), WorkspaceError> {
        let path = Path::new("workspaces")
            .join(&input.epic_key)
            .join("repos")
            .join(&input.service_id);
        info!(
            "Opening service '{}' with Cursor: {}",
            input.service_id,
            path.display()
        );
        cmd("cursor", &[path.to_string_lossy().into_owned()])
            .run()
            .map_err(|e| {
                WorkspaceError::editor("cursor", format!("Failed to open service: {}", e))
            })?;
        Ok(())
    }
}

pub struct VSCodeEditorAdapter;

#[async_trait]
impl EditorAdapter for VSCodeEditorAdapter {
    fn id(&self) -> &'static str {
        "vscode"
    }

    fn label(&self) -> &'static str {
        "VS Code"
    }

    async fn is_available(&self) -> Result<bool, WorkspaceError> {
        let result = cmd("code", &["--version"])
            .stdout_null()
            .stderr_null()
            .run();
        Ok(result.is_ok())
    }

    async fn open_workspace(&self, input: OpenWorkspaceInput) -> Result<(), WorkspaceError> {
        let path = Path::new("workspaces")
            .join(&input.epic_key)
            .join(format!("{}.code-workspace", input.epic_key));
        info!("Opening workspace with VS Code: {}", path.display());
        cmd("code", &[path.to_string_lossy().into_owned()])
            .run()
            .map_err(|e| {
                WorkspaceError::editor("vscode", format!("Failed to open workspace: {}", e))
            })?;
        Ok(())
    }

    async fn open_service(&self, input: OpenServiceInput) -> Result<(), WorkspaceError> {
        let path = Path::new("workspaces")
            .join(&input.epic_key)
            .join("repos")
            .join(&input.service_id);
        info!(
            "Opening service '{}' with VS Code: {}",
            input.service_id,
            path.display()
        );
        cmd("code", &[path.to_string_lossy().into_owned()])
            .run()
            .map_err(|e| {
                WorkspaceError::editor("vscode", format!("Failed to open service: {}", e))
            })?;
        Ok(())
    }
}

pub struct ZedEditorAdapter;

#[async_trait]
impl EditorAdapter for ZedEditorAdapter {
    fn id(&self) -> &'static str {
        "zed"
    }

    fn label(&self) -> &'static str {
        "Zed"
    }

    async fn is_available(&self) -> Result<bool, WorkspaceError> {
        let result = cmd("zed", &["--version"]).stdout_null().stderr_null().run();
        Ok(result.is_ok())
    }

    async fn open_workspace(&self, input: OpenWorkspaceInput) -> Result<(), WorkspaceError> {
        let path = Path::new("workspaces").join(&input.epic_key);
        info!("Opening workspace with Zed: {}", path.display());
        cmd("zed", &[path.to_string_lossy().into_owned()])
            .run()
            .map_err(|e| {
                WorkspaceError::editor("zed", format!("Failed to open workspace: {}", e))
            })?;
        Ok(())
    }

    async fn open_service(&self, input: OpenServiceInput) -> Result<(), WorkspaceError> {
        let path = Path::new("workspaces")
            .join(&input.epic_key)
            .join("repos")
            .join(&input.service_id);
        info!(
            "Opening service '{}' with Zed: {}",
            input.service_id,
            path.display()
        );
        cmd("zed", &[path.to_string_lossy().into_owned()])
            .run()
            .map_err(|e| WorkspaceError::editor("zed", format!("Failed to open service: {}", e)))?;
        Ok(())
    }
}

pub struct VimEditorAdapter;

#[async_trait]
impl EditorAdapter for VimEditorAdapter {
    fn id(&self) -> &'static str {
        "vim"
    }

    fn label(&self) -> &'static str {
        "Vim"
    }

    async fn is_available(&self) -> Result<bool, WorkspaceError> {
        let result = cmd("vim", &["--version"]).stdout_null().stderr_null().run();
        Ok(result.is_ok())
    }

    async fn open_workspace(&self, input: OpenWorkspaceInput) -> Result<(), WorkspaceError> {
        let path = Path::new("workspaces").join(&input.epic_key);
        info!("Opening workspace with Vim: {}", path.display());
        let mut child = std::process::Command::new("vim")
            .arg(path.to_string_lossy().into_owned())
            .spawn()
            .map_err(|e| WorkspaceError::editor("vim", format!("Failed to spawn vim: {}", e)))?;
        let status = child
            .wait()
            .map_err(|e| WorkspaceError::editor("vim", format!("Vim failed to exit: {}", e)))?;
        if !status.success() {
            return Err(WorkspaceError::editor(
                "vim",
                "Vim exited with non-zero status",
            ));
        }
        Ok(())
    }

    async fn open_service(&self, input: OpenServiceInput) -> Result<(), WorkspaceError> {
        let path = Path::new("workspaces")
            .join(&input.epic_key)
            .join("repos")
            .join(&input.service_id);
        info!(
            "Opening service '{}' with Vim: {}",
            input.service_id,
            path.display()
        );
        let mut child = std::process::Command::new("vim")
            .arg(path.to_string_lossy().into_owned())
            .spawn()
            .map_err(|e| WorkspaceError::editor("vim", format!("Failed to spawn vim: {}", e)))?;
        let status = child
            .wait()
            .map_err(|e| WorkspaceError::editor("vim", format!("Vim failed to exit: {}", e)))?;
        if !status.success() {
            return Err(WorkspaceError::editor(
                "vim",
                "Vim exited with non-zero status",
            ));
        }
        Ok(())
    }
}

// ==========================================
// AI Command: editor.open
// ==========================================

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct EditorOpenInput {
    pub epic_key: String,
    pub service_id: Option<String>,
    pub editor: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct StatusOutput {
    pub success: bool,
    pub message: String,
}

pub struct EditorOpenCommand;

#[async_trait]
impl AiCommand for EditorOpenCommand {
    const ID: &'static str = "editor.open";
    const DESCRIPTION: &'static str = "Open the workspace or a specific service in an editor.";
    type Input = EditorOpenInput;
    type Output = StatusOutput;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError> {
        let editor_id = input
            .editor
            .or_else(|| {
                let ws_path = ctx
                    .workspace_root
                    .join("workspaces")
                    .join(&input.epic_key)
                    .join("workspace.yaml");
                if ws_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(ws_path) {
                        if let Ok(ws) = serde_yaml::from_str::<ws_core::models::Workspace>(&content)
                        {
                            return Some(ws.editor);
                        }
                    }
                }
                None
            })
            .unwrap_or_else(|| ctx.config.editor.default.clone());

        let adapter = ctx.editor_adapters.get(&editor_id).ok_or_else(|| {
            WorkspaceError::NotFound(format!(
                "Editor adapter '{}' not found/registered",
                editor_id
            ))
        })?;

        if let Some(service_id) = input.service_id {
            adapter
                .open_service(OpenServiceInput {
                    epic_key: input.epic_key.clone(),
                    service_id,
                })
                .await?;
        } else {
            adapter
                .open_workspace(OpenWorkspaceInput {
                    epic_key: input.epic_key.clone(),
                })
                .await?;
        }

        Ok(StatusOutput {
            success: true,
            message: format!("Opened in editor '{}'.", editor_id),
        })
    }
}
