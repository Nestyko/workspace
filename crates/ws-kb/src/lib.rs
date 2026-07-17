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

use include_dir::{include_dir, Dir, DirEntry};
use std::fs;
use std::path::Path;
use ws_core::error::WorkspaceError;

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
    // `Refreshed` (explicit --reset) lands with the `ws kb init --reset`
    // ticket, which also adds the `reset` parameter and its test.
}

/// A single asset's path (relative to the KB root) and its scaffold outcome.
#[derive(Debug, Clone)]
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

/// Scaffolds the knowledge-base tree under `<root>/catalog/knowledge/`.
///
/// **Skip-by-default:** existing files are preserved (status
/// [`AssetStatus::Skipped`]) and only missing dirs/seed files are created
/// (status [`AssetStatus::Written`]). Returns a [`ScaffoldReport`] recording
/// what was written/skipped.
///
/// (The `reset` parameter for explicit single-asset refresh lands with the
/// `ws kb init --reset` ticket; this skeleton delivers scaffold-all only.)
pub fn scaffold(root: &Path) -> Result<ScaffoldReport, WorkspaceError> {
    let dest_root = root.join("catalog").join("knowledge");
    let mut report = ScaffoldReport::default();
    extract_dir(&KB_ASSETS, &dest_root, &mut report)?;
    Ok(report)
}

/// Recursively writes a [`Dir`]'s entries to `dest_root`, skipping files that
/// already exist. Every [`DirEntry::path`] is relative to the embedded root,
/// so `dest_root` stays the join base at every depth.
fn extract_dir(dir: &Dir, dest_root: &Path, report: &mut ScaffoldReport) -> Result<(), WorkspaceError> {
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
        let report = scaffold(tmp.path()).unwrap();

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
            report.assets.iter().all(|a| a.status == AssetStatus::Written),
            "fresh scaffold should mark every asset as Written"
        );
    }

    #[test]
    fn scaffold_output_matches_embedded_bytes_exactly() {
        let tmp = TempDir::new().unwrap();
        let _ = scaffold(tmp.path()).unwrap();

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
        let first = scaffold(tmp.path()).unwrap();
        assert!(first.assets.iter().all(|a| a.status == AssetStatus::Written));

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
        let second = scaffold(tmp.path()).unwrap();
        assert_eq!(
            second.assets.len(),
            first.assets.len(),
            "re-run should report the same asset set as the first run"
        );
        assert!(
            second.assets.iter().all(|a| a.status == AssetStatus::Skipped),
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

        let report = scaffold(tmp.path()).unwrap();

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
            stdfs::read(kb_root.join(pre_existing_rel)).unwrap().as_slice(),
            user_content,
            "pre-existing user content must be byte-for-byte unchanged"
        );

        // And the previously-missing files now exist and match embedded bytes.
        for (rel, embedded_bytes) in &embedded {
            if rel == &pre_existing_rel {
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
}
