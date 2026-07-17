//! `ws kb` — knowledge-base scaffold & lint (library seam).
//!
//! Pure functions over a filesystem root, mirroring the idiom of
//! `ws-catalog`'s `ensure_catalog_dirs` / `add_knowledge` functions but
//! establishing the workspace's first `tempfile::TempDir` test pattern.
//!
//! The canonical knowledge-base asset tree physically lives under
//! `crates/ws-cli/assets/catalog-knowledge/` (the restructure ADR's single
//! canonical embedded-asset location, Decision 3) and is embedded into this
//! crate at build time via `include_dir!`.
//!
//! ## Test pattern (for follow-on tickets: `lint`, `idempotency`)
//!
//! Tests live at the library-function seam (`scaffold`/`lint`) over a
//! `tempfile::TempDir` root — no CLI, no real workspace. The independent
//! source of truth is the embedded `KB_ASSETS` tree: tests walk it to derive
//! expected paths/bytes, then compare against what landed on disk. Mirror this
//! shape in `lint` tests (build fixture wikis under a TempDir, assert
//! findings).

use include_dir::{include_dir, Dir, DirEntry, File};
use std::fs;
use std::path::Path;
use ws_core::error::WorkspaceError;

pub mod lint;
pub use lint::{lint, FindingType, LintFinding, Severity};

/// The canonical embedded knowledge-base asset tree.
///
/// Physically rooted at `crates/ws-cli/assets/catalog-knowledge/`; embedded
/// into the `ws-kb` binary at build time. Every entry's [`DirEntry::path`] is
/// relative to this root.
pub static KB_ASSETS: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../ws-cli/assets/catalog-knowledge");

/// Per-asset outcome recorded by [`scaffold`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetStatus {
    /// Newly created (the file did not exist before).
    Written,
    /// Left untouched (the file already existed; skip-by-default).
    Skipped,
    /// Explicitly rewritten via `ws kb init --reset <asset>`.
    Refreshed,
}

/// Identifies one of the embedded assets under `catalog-knowledge/`.
///
/// The variant's name is a Rust identifier; the CLI-facing token is
/// [`AssetId::relative_path`], which is the file path relative to the
/// embedded KB root (e.g. `SCHEMA.md`, `wiki/index.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetId {
    Gitkeep,
    Readme,
    Schema,
    RawGitignore,
    RawReadme,
    RawAssetsGitkeep,
    WikiTemplate,
    WikiIndex,
    WikiLog,
    WikiEntitiesGitkeep,
    WikiSourcesGitkeep,
    WikiSynthesisGitkeep,
    WikiTopicsGitkeep,
}

impl AssetId {
    /// Path of this asset relative to the embedded `catalog-knowledge/` root.
    /// This is the exact, case-sensitive `--reset <asset>` token.
    pub const fn relative_path(self) -> &'static str {
        match self {
            AssetId::Gitkeep => ".gitkeep",
            AssetId::Readme => "README.md",
            AssetId::Schema => "SCHEMA.md",
            AssetId::RawGitignore => "raw/.gitignore",
            AssetId::RawReadme => "raw/README.md",
            AssetId::RawAssetsGitkeep => "raw/assets/.gitkeep",
            AssetId::WikiTemplate => "wiki/_template.md",
            AssetId::WikiIndex => "wiki/index.md",
            AssetId::WikiLog => "wiki/log.md",
            AssetId::WikiEntitiesGitkeep => "wiki/entities/.gitkeep",
            AssetId::WikiSourcesGitkeep => "wiki/sources/.gitkeep",
            AssetId::WikiSynthesisGitkeep => "wiki/synthesis/.gitkeep",
            AssetId::WikiTopicsGitkeep => "wiki/topics/.gitkeep",
        }
    }

    /// All embedded assets, in declaration order.
    pub const ALL: &'static [AssetId] = &[
        AssetId::Gitkeep,
        AssetId::Readme,
        AssetId::Schema,
        AssetId::RawGitignore,
        AssetId::RawReadme,
        AssetId::RawAssetsGitkeep,
        AssetId::WikiTemplate,
        AssetId::WikiIndex,
        AssetId::WikiLog,
        AssetId::WikiEntitiesGitkeep,
        AssetId::WikiSourcesGitkeep,
        AssetId::WikiSynthesisGitkeep,
        AssetId::WikiTopicsGitkeep,
    ];

    /// Returns all valid `--reset` tokens.
    pub fn valid_names() -> Vec<String> {
        Self::ALL
            .iter()
            .map(|a| a.relative_path().to_string())
            .collect()
    }

    /// Parses a `--reset` token into an [`AssetId`].
    pub fn from_arg(s: &str) -> Result<Self, UnknownAsset> {
        for &id in Self::ALL {
            if id.relative_path() == s {
                return Ok(id);
            }
        }
        Err(UnknownAsset {
            requested: s.to_string(),
            valid: Self::valid_names(),
        })
    }
}

/// Error returned when a requested `--reset` asset name is not known.
#[derive(Debug, Clone)]
pub struct UnknownAsset {
    pub requested: String,
    pub valid: Vec<String>,
}

impl std::fmt::Display for UnknownAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unknown asset '{}'. Valid names:\n{}",
            self.requested,
            self.valid
                .iter()
                .map(|v| format!("  - {v}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl std::error::Error for UnknownAsset {}

/// A single asset's path (relative to the KB root) and its scaffold outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetReport {
    pub path: String,
    pub status: AssetStatus,
}

/// Report returned by [`scaffold`]: one [`AssetReport`] per embedded file, in
/// iteration order.
#[derive(Debug, Clone, Default)]
pub struct ScaffoldReport {
    pub assets: Vec<AssetReport>,
}

/// Runs the `ws kb init` workflow and returns a human-readable summary
/// (header, one line per asset, and a tally).
///
/// This is the KB crate's single entry point for the CLI: it parses the
/// `--reset` token, dispatches to [`scaffold`], and renders the report text.
/// The caller (`main.rs`) owns only the I/O — printing the returned string —
/// so the CLI stays a thin dispatcher and KB-specific orchestration lives
/// here next to the data it acts on.
///
/// `reset` is the raw `--reset <asset>` token (a path relative to the
/// embedded KB root). [`None`] means the default skip-by-default scaffold.
pub fn run_init(root: &Path, reset: Option<&str>) -> Result<String, WorkspaceError> {
    let (report, header) = match reset {
        Some(name) => {
            let id = AssetId::from_arg(name).map_err(|e| WorkspaceError::UnknownAsset {
                requested: e.requested,
                valid: e.valid,
            })?;
            (scaffold(root, Some(id))?, "Knowledge base reset:")
        }
        None => (scaffold(root, None)?, "Knowledge base scaffold:"),
    };

    let mut out = String::from(header);
    out.push('\n');
    let mut written = 0;
    let mut skipped = 0;
    for a in &report.assets {
        out.push_str(&format!(
            "  {:<10} {}\n",
            format!("{:?}", a.status).to_lowercase(),
            a.path
        ));
        match a.status {
            AssetStatus::Written => written += 1,
            AssetStatus::Skipped => skipped += 1,
            AssetStatus::Refreshed => {}
        }
    }
    if reset.is_some() {
        out.push_str("\n1 refreshed.");
    } else {
        out.push_str(&format!("\n{} written, {} skipped.", written, skipped));
    }
    Ok(out)
}

/// Scaffolds the knowledge-base tree under `<root>/catalog/knowledge/`.
///
/// **Skip-by-default:** existing files are preserved (status
/// [`AssetStatus::Skipped`]) and only missing dirs/seed files are created
/// (status [`AssetStatus::Written`]). Returns a [`ScaffoldReport`] recording
/// what was written/skipped.
///
/// If `reset` is [`Some`], only the named asset is rewritten; the report
/// contains exactly one entry with status [`AssetStatus::Refreshed`] and no
/// other files are touched.
pub fn scaffold(root: &Path, reset: Option<AssetId>) -> Result<ScaffoldReport, WorkspaceError> {
    let dest_root = root.join("catalog").join("knowledge");
    let mut report = ScaffoldReport::default();

    if let Some(id) = reset {
        let rel = id.relative_path();
        let file = KB_ASSETS
            .get_file(rel)
            .expect("AssetId invariant: variant has no embedded file");
        let dest_path = dest_root.join(rel);
        refresh_one(file, &dest_path)?;
        report.assets.push(AssetReport {
            path: rel.to_string(),
            status: AssetStatus::Refreshed,
        });
        return Ok(report);
    }

    extract_dir(&KB_ASSETS, &dest_root, &mut report)?;
    Ok(report)
}

/// Unconditionally writes a single embedded file to disk, creating parent
/// directories as needed. Used by the `--reset` code path.
fn refresh_one(file: &File, dest_path: &Path) -> Result<(), WorkspaceError> {
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(dest_path, file.contents())?;
    Ok(())
}

/// Recursively writes a [`Dir`]'s entries to `dest_root`, skipping files that
/// already exist. Every [`DirEntry::path`] is relative to the embedded root,
/// so `dest_root` stays the join base at every depth.
fn extract_dir(
    dir: &Dir,
    dest_root: &Path,
    report: &mut ScaffoldReport,
) -> Result<(), WorkspaceError> {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(d) => {
                fs::create_dir_all(dest_root.join(d.path()))?;
                extract_dir(d, dest_root, report)?;
            }
            DirEntry::File(f) => {
                let rel = f.path().to_string_lossy().into_owned();
                let dest_path = dest_root.join(&rel);
                let status = if dest_path.exists() {
                    AssetStatus::Skipped
                } else {
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&dest_path, f.contents())?;
                    AssetStatus::Written
                };
                report.assets.push(AssetReport { path: rel, status });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as stdfs;
    use tempfile::TempDir;

    /// Recursively collects `(relative_path, embedded_bytes)` for every file
    /// in the embedded KB tree. The embedded bytes are the independent source
    /// of truth the tests compare disk output against.
    fn embedded_files() -> Vec<(String, &'static [u8])> {
        let mut out = Vec::new();
        collect(&KB_ASSETS, &mut out);
        return out;

        fn collect<'a>(dir: &Dir<'a>, out: &mut Vec<(String, &'a [u8])>) {
            for entry in dir.entries() {
                match entry {
                    DirEntry::File(f) => {
                        out.push((f.path().to_string_lossy().into_owned(), f.contents()));
                    }
                    DirEntry::Dir(d) => collect(d, out),
                }
            }
        }
    }

    #[test]
    fn scaffold_writes_full_tree_from_embedded_assets() {
        let tmp = TempDir::new().unwrap();
        let report = scaffold(tmp.path(), None).unwrap();

        let embedded = embedded_files();
        // Sanity: the embedded tree carries a meaningful set of assets
        // (guards against an accidentally-empty embed).
        assert!(
            embedded.len() >= 10,
            "expected at least 10 embedded assets, found {}",
            embedded.len()
        );

        // Every embedded file exists on disk under <tmp>/catalog/knowledge/.
        for (rel, _) in &embedded {
            let on_disk = tmp.path().join("catalog").join("knowledge").join(rel);
            assert!(
                on_disk.is_file(),
                "expected scaffolded file at catalog/knowledge/{rel}, not found"
            );
        }

        // Every file was newly written (fresh tempdir).
        assert_eq!(
            report.assets.len(),
            embedded.len(),
            "report should record one entry per embedded file"
        );
        assert!(
            report
                .assets
                .iter()
                .all(|a| a.status == AssetStatus::Written),
            "fresh scaffold should mark every asset as Written"
        );
    }

    #[test]
    fn scaffold_output_matches_embedded_bytes_exactly() {
        let tmp = TempDir::new().unwrap();
        let _ = scaffold(tmp.path(), None).unwrap();

        let kb_root = tmp.path().join("catalog").join("knowledge");
        for (rel, embedded_bytes) in embedded_files() {
            let on_disk = kb_root.join(&rel);
            let disk_bytes = stdfs::read(&on_disk).unwrap();
            assert_eq!(
                disk_bytes.as_slice(),
                embedded_bytes,
                "byte mismatch for catalog/knowledge/{rel}"
            );
        }
    }

    #[test]
    fn scaffold_skips_existing_files_on_rerun() {
        // First run: full scaffold.
        let tmp = TempDir::new().unwrap();
        let first = scaffold(tmp.path(), None).unwrap();
        assert!(first
            .assets
            .iter()
            .all(|a| a.status == AssetStatus::Written));

        // Simulate user edits: overwrite a couple of scaffolded files with
        // bespoke content. A re-run must NOT touch them.
        let kb_root = tmp.path().join("catalog").join("knowledge");
        let edited_rel = "wiki/index.md";
        let edited_path = kb_root.join(edited_rel);
        let user_content = b"# My bespoke wiki index\nNothing to see here.\n";
        stdfs::write(&edited_path, user_content).unwrap();
        let other_edited = kb_root.join("SCHEMA.md");
        let other_user_content = b"# bespoke schema\n";
        stdfs::write(&other_edited, other_user_content).unwrap();

        // Second run: every asset should be Skipped.
        let second = scaffold(tmp.path(), None).unwrap();
        assert_eq!(
            second.assets.len(),
            first.assets.len(),
            "re-run should report the same asset set as the first run"
        );
        assert!(
            second
                .assets
                .iter()
                .all(|a| a.status == AssetStatus::Skipped),
            "re-run on a fully-populated tree should mark every asset as Skipped"
        );

        // User-edited content is byte-for-byte preserved.
        assert_eq!(
            stdfs::read(&edited_path).unwrap().as_slice(),
            user_content,
            "user-edited wiki/index.md must be untouched on re-run"
        );
        assert_eq!(
            stdfs::read(&other_edited).unwrap().as_slice(),
            other_user_content,
            "user-edited SCHEMA.md must be untouched on re-run"
        );
    }

    #[test]
    fn scaffold_fills_only_missing_files_on_partial_tree() {
        let tmp = TempDir::new().unwrap();
        // Pre-populate one file with user content and leave the rest missing.
        let kb_root = tmp.path().join("catalog").join("knowledge");
        stdfs::create_dir_all(kb_root.join("wiki")).unwrap();
        let pre_existing_rel = "wiki/index.md";
        let user_content = b"# pre-existing user content\n";
        stdfs::write(kb_root.join(pre_existing_rel), user_content).unwrap();

        let report = scaffold(tmp.path(), None).unwrap();

        // Every embedded file appears in the report.
        let embedded = embedded_files();
        assert_eq!(report.assets.len(), embedded.len());

        // The pre-existing one is Skipped; all others are Written.
        let pre = report
            .assets
            .iter()
            .find(|a| a.path == pre_existing_rel)
            .unwrap_or_else(|| panic!("pre-existing asset {pre_existing_rel} missing from report"));
        assert_eq!(pre.status, AssetStatus::Skipped);
        assert!(
            report
                .assets
                .iter()
                .filter(|a| a.path != pre_existing_rel)
                .all(|a| a.status == AssetStatus::Written),
            "all other assets should be Written (gaps filled)"
        );

        // Pre-existing user content untouched.
        assert_eq!(
            stdfs::read(kb_root.join(pre_existing_rel))
                .unwrap()
                .as_slice(),
            user_content,
            "pre-existing user content must be byte-for-byte unchanged"
        );

        // And the previously-missing files now exist and match embedded bytes.
        for (rel, embedded_bytes) in &embedded {
            if rel == pre_existing_rel {
                continue;
            }
            let on_disk = kb_root.join(rel);
            assert_eq!(
                stdfs::read(&on_disk).unwrap().as_slice(),
                *embedded_bytes,
                "missing file {rel} should now match embedded bytes"
            );
        }
    }

    /// Snapshot of all file (rel_path, bytes) pairs under the KB root.
    fn snapshot_kb(kb_root: &Path) -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        for (rel, _bytes) in embedded_files() {
            let path = kb_root.join(&rel);
            let disk = if path.exists() {
                stdfs::read(&path).unwrap()
            } else {
                Vec::new()
            };
            out.push((rel, disk));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    #[test]
    fn reset_rewrites_exactly_the_named_asset() {
        let tmp = TempDir::new().unwrap();
        let _ = scaffold(tmp.path(), None).unwrap();
        let kb_root = tmp.path().join("catalog").join("knowledge");

        let target = AssetId::Schema;
        let target_rel = target.relative_path();
        let target_path = kb_root.join(target_rel);
        let before = snapshot_kb(&kb_root);

        // Mutate the target file on disk.
        let mutant = b"# mutated schema\n";
        stdfs::write(&target_path, mutant).unwrap();

        // Reset only the target asset.
        let report = scaffold(tmp.path(), Some(target)).unwrap();
        assert_eq!(
            report.assets,
            vec![AssetReport {
                path: target_rel.to_string(),
                status: AssetStatus::Refreshed,
            }]
        );

        // Target now matches embedded bytes; everything else is byte-identical
        // to the pre-reset snapshot.
        let embedded_bytes = embedded_files()
            .into_iter()
            .find(|(rel, _)| rel == target_rel)
            .map(|(_, bytes)| bytes)
            .unwrap();
        assert_eq!(
            stdfs::read(&target_path).unwrap().as_slice(),
            embedded_bytes,
            "reset asset should be rewritten with embedded bytes"
        );

        let after = snapshot_kb(&kb_root);
        for ((rel_before, bytes_before), (rel_after, bytes_after)) in
            before.iter().zip(after.iter())
        {
            assert_eq!(rel_before, rel_after);
            if rel_before != target_rel {
                assert_eq!(
                    bytes_before, bytes_after,
                    "non-reset file {rel_before} must be untouched"
                );
            }
        }
    }

    #[test]
    fn reset_touches_nothing_beside_the_named_asset() {
        let tmp = TempDir::new().unwrap();
        let _ = scaffold(tmp.path(), None).unwrap();
        let kb_root = tmp.path().join("catalog").join("knowledge");

        let target = AssetId::WikiIndex;
        let target_rel = target.relative_path();
        let before = snapshot_kb(&kb_root);

        let report = scaffold(tmp.path(), Some(target)).unwrap();
        assert_eq!(
            report.assets,
            vec![AssetReport {
                path: target_rel.to_string(),
                status: AssetStatus::Refreshed,
            }]
        );

        let after = snapshot_kb(&kb_root);
        for ((rel_before, bytes_before), (rel_after, bytes_after)) in
            before.iter().zip(after.iter())
        {
            assert_eq!(rel_before, rel_after);
            assert_eq!(
                bytes_before, bytes_after,
                "reset without mutation should still leave {rel_before} byte-identical"
            );
        }
    }

    #[test]
    fn reset_is_idempotent_on_same_embedded_bytes() {
        let tmp = TempDir::new().unwrap();
        let _ = scaffold(tmp.path(), None).unwrap();

        let target = AssetId::Readme;
        let target_rel = target.relative_path();

        let first = scaffold(tmp.path(), Some(target)).unwrap();
        assert_eq!(
            first.assets,
            vec![AssetReport {
                path: target_rel.to_string(),
                status: AssetStatus::Refreshed,
            }]
        );

        let first_bytes = stdfs::read(
            tmp.path()
                .join("catalog")
                .join("knowledge")
                .join(target_rel),
        )
        .unwrap();

        let second = scaffold(tmp.path(), Some(target)).unwrap();
        assert_eq!(
            second.assets,
            vec![AssetReport {
                path: target_rel.to_string(),
                status: AssetStatus::Refreshed,
            }]
        );

        let second_bytes = stdfs::read(
            tmp.path()
                .join("catalog")
                .join("knowledge")
                .join(target_rel),
        )
        .unwrap();
        assert_eq!(first_bytes, second_bytes);
    }

    #[test]
    fn from_arg_rejects_unknown_name_lists_valid_names() {
        let err = AssetId::from_arg("does-not-exist.md").unwrap_err();
        assert_eq!(err.requested, "does-not-exist.md");
        assert_eq!(err.valid, AssetId::valid_names());
        assert_eq!(err.valid.len(), 13);
        assert!(err.to_string().contains("does-not-exist.md"));
        assert!(err.to_string().contains("SCHEMA.md"));
    }

    #[test]
    fn run_init_default_renders_header_and_tally() {
        let tmp = TempDir::new().unwrap();
        let summary = run_init(tmp.path(), None).unwrap();

        assert!(
            summary.starts_with("Knowledge base scaffold:\n"),
            "default run should open with the scaffold header: {summary:?}"
        );
        let embedded = embedded_files();
        assert_eq!(
            summary
                .lines()
                .filter(|l| !l.is_empty())
                .count()
                .saturating_sub(2),
            embedded.len(),
            "summary should have one asset line per embedded file (plus header + tally)"
        );
        assert!(
            summary.ends_with(&format!("\n{} written, {} skipped.", embedded.len(), 0)),
            "fresh scaffold should tally all written, none skipped: {summary:?}"
        );
    }

    #[test]
    fn run_init_reset_renders_refresh_header_and_tally() {
        let tmp = TempDir::new().unwrap();
        let _ = scaffold(tmp.path(), None).unwrap();

        let summary = run_init(tmp.path(), Some("SCHEMA.md")).unwrap();
        assert!(
            summary.starts_with("Knowledge base reset:\n"),
            "reset run should open with the reset header: {summary:?}"
        );
        assert!(
            summary.contains("refreshed"),
            "reset summary should mention a refresh: {summary:?}"
        );
        assert!(
            summary.contains("SCHEMA.md"),
            "reset summary should name the refreshed asset: {summary:?}"
        );
        assert!(
            summary.ends_with("\n1 refreshed."),
            "reset summary should end with the one-refreshed tally: {summary:?}"
        );
    }

    #[test]
    fn run_init_reset_unknown_name_errors_propagating_valid_list() {
        let tmp = TempDir::new().unwrap();
        let err = run_init(tmp.path(), Some("bogus.md")).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("bogus.md"),
            "error must name the request: {msg}"
        );
        assert!(
            msg.contains("SCHEMA.md"),
            "error must list a valid name: {msg}"
        );
    }
}
