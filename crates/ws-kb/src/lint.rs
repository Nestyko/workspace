//! Wiki lint: deterministic health check over `wiki/**/*.md`.
//!
//! The mechanical subset of wiki lint, per the restructure ADR / feature spec:
//! **orphans** (wiki pages with no inbound `[[wikilinks]]`) and **broken
//! `[[wikilinks]]`** (links whose target slug resolves to no page). The
//! judgment subset (contradictions, stale claims, missing cross-references)
//! stays a SCHEMA-defined agent operation.
//!
//! ## `[[wikilink]]` resolution rule (from the feature spec)
//! A link target resolves if the slug (filename without `.md`) exists anywhere
//! under `wiki/`, ignoring the category folder — i.e. `[[revenue]]` resolves to
//! `wiki/topics/revenue.md` or `wiki/entities/revenue.md`. Broken = no page
//! with that slug exists. Orphan = a wiki page whose slug is never the target of
//! any `[[wikilink]]` across the wiki.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use ws_core::error::WorkspaceError;

/// What kind of mechanical lint finding this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingType {
    /// A wiki page with no inbound `[[wikilinks]]`.
    Orphan,
    /// A `[[wikilink]]` whose target slug resolves to no page.
    BrokenLink,
}

impl FindingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingType::Orphan => "orphan",
            FindingType::BrokenLink => "broken-link",
        }
    }
}

/// Severity. MVP scope is minimal — every finding is a warning. The enum is
/// the extension point named by the feature spec (which may introduce
/// `Error`/`Info`); the CLI report format requires a severity column per the
/// ticket's acceptance criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        "warning"
    }
}

/// A single mechanical lint finding.
#[derive(Debug, Clone)]
pub struct LintFinding {
    /// Path of the page the finding is about, relative to the wiki root
    /// (e.g. `topics/revenue.md`). For a broken link, the page containing the
    /// broken link.
    pub page: String,
    pub finding_type: FindingType,
    pub severity: Severity,
    pub message: String,
}

impl LintFinding {
    fn new(page: String, finding_type: FindingType, message: String) -> Self {
        LintFinding {
            page,
            finding_type,
            severity: Severity::Warning,
            message,
        }
    }
}

/// Runs the mechanical wiki lint over `<root>/catalog/knowledge/wiki/`.
///
/// Returns one [`LintFinding`] per orphan page and per broken `[[wikilink]]`.
/// If the wiki directory does not exist, returns an empty vec (nothing to lint).
pub fn lint(root: &Path) -> Result<Vec<LintFinding>, WorkspaceError> {
    let wiki_root = root.join("catalog").join("knowledge").join("wiki");
    if !wiki_root.is_dir() {
        return Ok(Vec::new());
    }

    // 1. Collect every wiki page: slug -> relative path (folder-agnostic).
    //    Exclude `_`-prefixed files (templates) from the page set, but still
    //    scan them for outbound links.
    let mut pages: Vec<(String, PathBuf)> = Vec::new(); // (slug, rel_path)
    let mut all_md: Vec<PathBuf> = Vec::new(); // rel paths of every .md to scan
    collect_md_files(&wiki_root, &wiki_root, &mut pages, &mut all_md)?;

    let slug_set: HashSet<&str> = pages.iter().map(|(s, _)| s.as_str()).collect();

    // 2. Walk every .md, extract [[wikilink]] targets, record broken links and
    //    collect the set of targeted slugs (for orphan detection).
    let mut targeted: HashSet<String> = HashSet::new();
    let mut broken: Vec<LintFinding> = Vec::new();

    for rel in &all_md {
        let abs = wiki_root.join(rel);
        let contents = fs::read_to_string(&abs)?;
        let page_rel = rel.to_string_lossy().into_owned();
        for target in extract_wikilink_targets(&contents) {
            targeted.insert(target.clone());
            if !slug_set.contains(target.as_str()) {
                broken.push(LintFinding::new(
                    page_rel.clone(),
                    FindingType::BrokenLink,
                    format!("[[{}]] resolves to no page", target),
                ));
            }
        }
    }

    // 3. Second pass: a page is an orphan only if no wikilink anywhere targets
    //    its slug. (Requires the full `targeted` set from pass 2.)
    let mut findings = broken;
    for (slug, rel) in &pages {
        if !targeted.contains(slug) {
            findings.push(LintFinding::new(
                rel.to_string_lossy().into_owned(),
                FindingType::Orphan,
                format!("page '{}' has no inbound [[wikilinks]]", slug),
            ));
        }
    }

    Ok(findings)
}

/// Recursively walks `dir`, populating `pages` (slug, rel_path) for non-template
/// .md files and `all_md` (rel_path) for every .md file (templates included).
///
/// `_`-prefixed files (templates) are scanned for outbound links but excluded
/// from the page set / orphan check — they are scaffolding, not concept pages
/// (per SCHEMA.md, `_template.md` is an empty starter, not wiki content).
fn collect_md_files(
    base: &Path,
    dir: &Path,
    pages: &mut Vec<(String, PathBuf)>,
    all_md: &mut Vec<PathBuf>,
) -> Result<(), WorkspaceError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(base, &path, pages, all_md)?;
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let rel = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            all_md.push(rel.clone());
            if !filename.starts_with('_') {
                let slug = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                pages.push((slug, rel));
            }
        }
    }
    Ok(())
}

/// Extracts the target slugs of every `[[wikilink]]` in `text`.
///
/// Handles Obsidian forms: `[[slug]]`, `[[slug|alias]]`, `[[slug#block]]`,
/// `[[topics/slug]]` (folder-agnostic — takes the last path component), and
/// combinations. The returned slug is the bare page identifier (no folder, no
/// block ref, no alias).
fn extract_wikilink_targets(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            // find closing ]]
            if let Some(end_rel) = text[i + 2..].find("]]") {
                let inner = &text[i + 2..i + 2 + end_rel];
                // strip alias (after |) and block ref (after #)
                let target = inner.split('|').next().unwrap_or(inner);
                let target = target.split('#').next().unwrap_or(target);
                // folder-agnostic: take the last path component
                let slug = target
                    .trim()
                    .rsplit('/')
                    .next()
                    .unwrap_or(target.trim())
                    .to_string();
                if !slug.is_empty() {
                    out.push(slug);
                }
                i += 2 + end_rel + 2;
                continue;
            }
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper: lay out a wiki under a tempdir's catalog/knowledge/wiki/ with
    /// the given `(rel_path, contents)` files. Returns the tempdir (caller
    /// keeps it alive) and its path.
    fn wiki_with(files: &[(&str, &str)]) -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_path_buf();
        let wiki = root.join("catalog").join("knowledge").join("wiki");
        for (rel, contents) in files {
            let p = wiki.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, contents).unwrap();
        }
        (tmp, root)
    }

    #[test]
    fn clean_wiki_yields_no_findings() {
        // revenue -> links to [[pricing]] ; pricing -> links to [[revenue]].
        // No orphans, no broken links.
        let (_tmp, root) = wiki_with(&[
            ("topics/revenue.md", "# Revenue\nSee [[pricing]].\n"),
            ("topics/pricing.md", "# Pricing\nDrives [[revenue]].\n"),
        ]);
        let findings = lint(&root).unwrap();
        assert!(
            findings.is_empty(),
            "clean wiki should yield no findings, got: {findings:?}"
        );
    }

    #[test]
    fn orphan_page_is_reported() {
        // revenue links to pricing, but nothing links to revenue → revenue is
        // an orphan. pricing has an inbound link → not an orphan.
        let (_tmp, root) = wiki_with(&[
            ("topics/revenue.md", "# Revenue\nSee [[pricing]].\n"),
            ("topics/pricing.md", "# Pricing\nNothing here.\n"),
        ]);
        let findings = lint(&root).unwrap();
        let orphans: Vec<_> = findings
            .iter()
            .filter(|f| f.finding_type == FindingType::Orphan)
            .collect();
        assert_eq!(orphans.len(), 1, "expected exactly one orphan: {findings:?}");
        assert_eq!(orphans[0].page, "topics/revenue.md");
    }

    #[test]
    fn broken_wikilink_is_reported() {
        // [[nonexistent]] resolves to no page.
        let (_tmp, root) = wiki_with(&[
            (
                "topics/revenue.md",
                "# Revenue\nSee [[pricing]] and [[nonexistent]].\n",
            ),
            ("topics/pricing.md", "# Pricing\nDrives [[revenue]].\n"),
        ]);
        let findings = lint(&root).unwrap();
        let broken: Vec<_> = findings
            .iter()
            .filter(|f| f.finding_type == FindingType::BrokenLink)
            .collect();
        assert_eq!(
            broken.len(),
            1,
            "expected exactly one broken link: {findings:?}"
        );
        assert_eq!(broken[0].page, "topics/revenue.md");
        assert!(broken[0].message.contains("nonexistent"));
    }

    #[test]
    fn slug_resolution_is_folder_agnostic() {
        // [[revenue]] is written in entities/, but the page lives in topics/.
        // Folder-agnostic resolution: the link resolves; no broken link.
        let (_tmp, root) = wiki_with(&[
            ("entities/customer-a.md", "# Customer A\nLoves [[revenue]].\n"),
            ("topics/revenue.md", "# Revenue\nSee [[customer-a]].\n"),
        ]);
        let findings = lint(&root).unwrap();
        let broken: Vec<_> = findings
            .iter()
            .filter(|f| f.finding_type == FindingType::BrokenLink)
            .collect();
        assert!(
            broken.is_empty(),
            "cross-folder wikilinks should resolve: {findings:?}"
        );
    }

    #[test]
    fn alias_and_block_refs_do_not_break_resolution() {
        // [[revenue|the money]] and [[pricing#section]] both resolve.
        let (_tmp, root) = wiki_with(&[
            (
                "topics/revenue.md",
                "# Revenue\nSee [[pricing#section]] and [[customer-a|the money]].\n",
            ),
            ("topics/pricing.md", "# Pricing\nDrives [[revenue|the money]].\n"),
            ("entities/customer-a.md", "# Customer A\nLoves [[revenue]].\n"),
        ]);
        let findings = lint(&root).unwrap();
        assert!(
            findings.is_empty(),
            "aliases and block refs should still resolve: {findings:?}"
        );
    }

    #[test]
    fn missing_wiki_dir_yields_no_findings() {
        let tmp = TempDir::new().unwrap();
        let findings = lint(tmp.path()).unwrap();
        assert!(findings.is_empty(), "missing wiki dir should lint clean");
    }

    #[test]
    fn extract_wikilink_targets_handles_forms() {
        let got = extract_wikilink_targets("a [[b]] c [[d|alias]] e [[f#block]] g [[h/i]]");
        assert_eq!(got, vec!["b", "d", "f", "i"]);
    }
}
