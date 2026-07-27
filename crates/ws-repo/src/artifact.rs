//! Shared resolution helpers for the Understand-Anything knowledge-graph artifact
//! (repo-init healthcheck Point #1).
//!
//! The artifact directory has migrated from the legacy `.understand-anything/` to
//! the preferred short form `.ua/`. These helpers centralize the "prefer `.ua/`,
//! fall back to `.understand-anything/`" rule so the healthcheck (#1 gate) and
//! `repo.understand.verify` agree on where the artifact lives. No forced migration
//! of existing repos — legacy dirs keep working; the skill prompts the customer to
//! migrate when a legacy dir is found.

use std::path::{Path, PathBuf};

/// Preferred artifact directory (current Understand-Anything default).
pub const PREFERRED_DIR: &str = ".ua";

/// Legacy artifact directory (older Understand-Anything default).
pub const LEGACY_DIR: &str = ".understand-anything";

/// Directories to probe, preferred first.
pub const ARTIFACT_DIRS: &[&str] = &[PREFERRED_DIR, LEGACY_DIR];

/// Artifact filename inside the directory.
pub const ARTIFACT_FILE: &str = "knowledge-graph.json";

/// Resolve the committed knowledge-graph artifact path.
///
/// Returns the first existing `<dir>/knowledge-graph.json` under `repo_root`,
/// probing `.ua/` (preferred) before the legacy `.understand-anything/`. `None`
/// if neither path exists on disk.
pub fn knowledge_graph_path(repo_root: &Path) -> Option<PathBuf> {
    ARTIFACT_DIRS
        .iter()
        .map(|dir| repo_root.join(dir).join(ARTIFACT_FILE))
        .find(|p| p.exists())
}

/// The directory name that the resolved artifact lives under, when present.
pub fn resolved_dir(repo_root: &Path) -> Option<&'static str> {
    // Derive from `knowledge_graph_path` rather than re-probing the same existence
    // chain, so the two helpers cannot drift apart.
    let path = knowledge_graph_path(repo_root)?;
    let dir = path.parent()?.file_name()?.to_str()?;
    match dir {
        PREFERRED_DIR => Some(PREFERRED_DIR),
        LEGACY_DIR => Some(LEGACY_DIR),
        _ => None,
    }
}

/// Whether a `.gitattributes` body carries the canonical diff-suppression lines
/// for the artifact under *either* directory.
///
/// Canonical lines (from `templates/understand-anything.gitattributes`):
/// ```gitattributes
/// .ua/knowledge-graph.json binary -diff linguist-generated
/// .ua/**                 linguist-generated
/// ```
/// The legacy form substitutes `.understand-anything/` for `.ua/`.
///
/// The meaningful discriminator is the `<dir>/knowledge-graph.json` token: a
/// `.gitattributes` that mentions the artifact path is treated as having the
/// suppression intent. This mirrors the permissive parity of the pre-widening
/// legacy check (which reduced to the same single-contains). Stricter validation
/// of the artifact itself (parses + Action green + PR merged) lives in
/// `repo.understand.verify`.
pub fn gitattributes_suppression_ok(content: &str) -> bool {
    ARTIFACT_DIRS
        .iter()
        .any(|dir| content.contains(&format!("{}/{}", dir, ARTIFACT_FILE)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    // --- knowledge_graph_path ---

    #[test]
    fn resolves_preferred_ua_dir_when_present() {
        let d = tempdir().unwrap();
        write(d.path(), ".ua/knowledge-graph.json", "{}");
        let got = knowledge_graph_path(d.path()).expect("should resolve");
        assert_eq!(got, d.path().join(".ua").join("knowledge-graph.json"));
        assert_eq!(resolved_dir(d.path()), Some(PREFERRED_DIR));
    }

    #[test]
    fn falls_back_to_legacy_dir() {
        let d = tempdir().unwrap();
        write(d.path(), ".understand-anything/knowledge-graph.json", "{}");
        let got = knowledge_graph_path(d.path()).expect("should resolve via legacy");
        assert_eq!(
            got,
            d.path()
                .join(".understand-anything")
                .join("knowledge-graph.json")
        );
        assert_eq!(resolved_dir(d.path()), Some(LEGACY_DIR));
    }

    #[test]
    fn prefers_ua_over_legacy_when_both_present() {
        let d = tempdir().unwrap();
        write(d.path(), ".understand-anything/knowledge-graph.json", "{}");
        write(d.path(), ".ua/knowledge-graph.json", "{}");
        let got = knowledge_graph_path(d.path()).expect("should resolve");
        assert_eq!(got, d.path().join(".ua").join("knowledge-graph.json"));
        assert_eq!(resolved_dir(d.path()), Some(PREFERRED_DIR));
    }

    #[test]
    fn returns_none_when_no_artifact() {
        let d = tempdir().unwrap();
        assert!(knowledge_graph_path(d.path()).is_none());
        assert_eq!(resolved_dir(d.path()), None);
    }

    // --- gitattributes_suppression_ok ---

    #[test]
    fn gitattributes_accepts_preferred_ua_lines() {
        let body =
            ".ua/knowledge-graph.json binary -diff linguist-generated\n.ua/** linguist-generated\n";
        assert!(gitattributes_suppression_ok(body));
    }

    #[test]
    fn gitattributes_accepts_legacy_dir_lines() {
        let body = ".understand-anything/knowledge-graph.json binary -diff linguist-generated\n.understand-anything/** linguist-generated\n";
        assert!(gitattributes_suppression_ok(body));
    }

    #[test]
    fn gitattributes_rejects_unrelated_body() {
        assert!(!gitattributes_suppression_ok("# nothing relevant here\n"));
    }

    #[test]
    fn gitattributes_rejects_unrelated_dir() {
        // A body that only mentions an unrelated artifact dir fails.
        assert!(!gitattributes_suppression_ok(
            "some-other/knowledge-graph.json binary -diff linguist-generated\n"
        ));
    }

    #[test]
    fn gitattributes_accepts_just_the_artifact_path_token() {
        // The meaningful discriminator is the `<dir>/knowledge-graph.json` token;
        // a single suppression-mention line satisfies the gate. This is the
        // permissive parity with the pre-widening legacy check.
        let body = ".ua/knowledge-graph.json binary -diff linguist-generated\n";
        assert!(gitattributes_suppression_ok(body));
        let legacy = ".understand-anything/knowledge-graph.json binary -diff\n";
        assert!(gitattributes_suppression_ok(legacy));
    }
}
