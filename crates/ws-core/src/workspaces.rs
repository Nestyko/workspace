use crate::error::WorkspaceError;
use crate::models::Workspace;
use std::fs;
use std::path::Path;

/// A workspace found on disk: `folder` is the derived directory name
/// (`TICKET-slug` or `slug`), `ws` is its parsed `workspace.yaml`.
#[derive(Debug, Clone)]
pub struct WorkspaceOnDisk {
    pub folder: String,
    pub ws: Workspace,
}

/// Scan `workspaces/*/workspace.yaml` (content-based, no manifest; spec Q7).
/// Skips subdirectories without a parseable `workspace.yaml`.
pub fn list_workspaces(root: &Path) -> Result<Vec<WorkspaceOnDisk>, WorkspaceError> {
    let dir = root.join("workspaces");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let folder = entry.file_name().to_string_lossy().into_owned();
        let yaml = entry.path().join("workspace.yaml");
        if !yaml.is_file() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&yaml) else {
            continue;
        };
        let Ok(ws) = serde_yaml::from_str::<Workspace>(&content) else {
            continue;
        };
        entries.push(WorkspaceOnDisk { folder, ws });
    }
    entries.sort_by(|a, b| a.folder.cmp(&b.folder));
    Ok(entries)
}

fn ambiguous(q: &str, folders: &[&str]) -> WorkspaceError {
    let list = folders
        .iter()
        .map(|f| format!("  {}", f))
        .collect::<Vec<_>>()
        .join("\n");
    WorkspaceError::Validation(format!(
        "Query '{}' matches multiple workspaces:\n{}\nUse a full id or ticket.",
        q, list
    ))
}

/// Resolve a workspace query to a unique folder + workspace by content.
///
/// Match order:
/// 1. exact: folder name, `Workspace.id` (slug) or `Workspace.ticket`
/// 2. fallback: folder-name prefix `{q}-` (e.g. `EPIC-123` → `EPIC-123-realtime-chat`)
///
/// More than one match in either level → `Validation` with the candidates.
pub fn resolve_workspace(root: &Path, q: &str) -> Result<WorkspaceOnDisk, WorkspaceError> {
    let q = q.trim();
    if q.is_empty() {
        return Err(WorkspaceError::Validation(
            "Workspace query must not be empty.".to_string(),
        ));
    }
    let entries = list_workspaces(root)?;

    let exact: Vec<&WorkspaceOnDisk> = entries
        .iter()
        .filter(|e| e.folder == q || e.ws.id == q || e.ws.ticket.as_deref() == Some(q))
        .collect();

    if exact.len() == 1 {
        return Ok(exact[0].clone());
    }
    if exact.len() > 1 {
        let folders = exact.iter().map(|e| e.folder.as_str()).collect::<Vec<_>>();
        return Err(ambiguous(q, &folders));
    }

    let prefix: Vec<&WorkspaceOnDisk> = entries
        .iter()
        .filter(|e| e.folder.starts_with(&format!("{}-", q)))
        .collect();

    if prefix.len() == 1 {
        return Ok(prefix[0].clone());
    }
    if prefix.len() > 1 {
        let folders = prefix.iter().map(|e| e.folder.as_str()).collect::<Vec<_>>();
        return Err(ambiguous(q, &folders));
    }

    Err(WorkspaceError::NotFound(format!(
        "No workspace matches '{}' under {}",
        q,
        root.join("workspaces").display()
    )))
}

/// Kebab-case slug for a feature title, e.g. "Realtime Chat Agent" → `realtime-chat-agent`.
/// Identity is the slug (`Workspace.id`); the folder name is derived, never identity.
pub fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            slug.push(ch);
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "workspace".to_string()
    } else {
        slug
    }
}

/// The derived folder name for a workspace: `TICKET-slug` when attached, else `slug`.
pub fn workspace_dir_name(ws: &Workspace) -> String {
    match &ws.ticket {
        Some(t) if !t.trim().is_empty() => format!("{}-{}", t.trim(), ws.id),
        _ => ws.id.clone(),
    }
}

/// `root/workspaces/<folder>` — the single choke point for the folder name.
pub fn get_workspace_dir(root: &Path, folder: &str) -> std::path::PathBuf {
    root.join("workspaces").join(folder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Workspace;
    use tempfile::TempDir;

    fn ws(id: &str, ticket: Option<&str>) -> Workspace {
        Workspace {
            id: id.to_string(),
            ticket: ticket.map(|t| t.to_string()),
            title: String::new(),
            description: String::new(),
            created_at: None,
            services: vec!["api".to_string()],
            base_branch: "main".to_string(),
            create_branches: true,
            editor: "cursor".to_string(),
            tasks: vec![],
        }
    }

    fn write_ws(root: &Path, folder: &str, w: &Workspace) {
        let dir = crate::workspaces::get_workspace_dir(root, folder);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("workspace.yaml"),
            serde_yaml::to_string(w).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn slugify_kebabs_titles() {
        assert_eq!(slugify("Realtime Chat Agent"), "realtime-chat-agent");
        assert_eq!(slugify("EPIC-123 Realtime!"), "epic-123-realtime");
        assert_eq!(slugify("  Mixed  CASE "), "mixed-case");
        assert_eq!(slugify("!!!"), "workspace");
    }

    #[test]
    fn folder_name_follows_ticket_then_slug() {
        assert_eq!(
            workspace_dir_name(&ws("realtime-chat", None)),
            "realtime-chat"
        );
        assert_eq!(
            workspace_dir_name(&ws("realtime-chat", Some("EPIC-123"))),
            "EPIC-123-realtime-chat"
        );
        // Blank ticket is treated as unticketed.
        assert_eq!(workspace_dir_name(&ws("a", Some("  "))), "a");
    }

    #[test]
    fn resolve_prefers_exact_id_over_component_prefix() {
        let tmp = TempDir::new().unwrap();
        write_ws(tmp.path(), "realtime-chat", &ws("realtime-chat", None));
        write_ws(tmp.path(), "calc", &ws("calc", None));
        // Exact id wins.
        let got = resolve_workspace(tmp.path(), "realtime-chat").unwrap();
        assert_eq!(got.folder, "realtime-chat");
        // Component prefix fallback: `q-` matches a full slug component.
        let got = resolve_workspace(tmp.path(), "realtime").unwrap();
        assert_eq!(got.folder, "realtime-chat");
        // And ticket-prefix fallback for an attached workspace.
        write_ws(
            tmp.path(),
            "EPIC-123-billing",
            &ws("billing", Some("EPIC-123")),
        );
        let got = resolve_workspace(tmp.path(), "EPIC-123").unwrap();
        assert_eq!(got.folder, "EPIC-123-billing");
    }

    #[test]
    fn resolve_matches_by_ticket_and_rejects_ambiguity() {
        let tmp = TempDir::new().unwrap();
        write_ws(tmp.path(), "EPIC-1-a", &ws("a", Some("EPIC-1")));
        write_ws(tmp.path(), "EPIC-2-b", &ws("b", Some("EPIC-2")));

        let got = resolve_workspace(tmp.path(), "EPIC-1").unwrap();
        assert_eq!(got.folder, "EPIC-1-a");

        // One shared id prefix → ambiguous.
        write_ws(tmp.path(), "extra", &ws("a", None));
        let err = resolve_workspace(tmp.path(), "a").unwrap_err();
        assert!(
            err.to_string().contains("matches multiple") || err.to_string().contains("ambiguous"),
            "ambiguous duplicate id should error: {}",
            err
        );
    }

    #[test]
    fn resolve_not_found_with_clear_message() {
        let tmp = TempDir::new().unwrap();
        let err = resolve_workspace(tmp.path(), "ghost").unwrap_err();
        assert!(err.to_string().contains("No workspace matches"), "{}", err);
    }
}
