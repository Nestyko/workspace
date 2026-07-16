//! Embedded canonical asset tree.
//!
//! Per the restructure ADR (Decision 3 + resolved Q3), `crates/ws-cli/assets/`
//! is the single canonical location for assets embedded into the binary at
//! build time. The `include_dir!` macro embeds the tree at compile time;
//! `Dir::entries()` iterates it for recursive `fs::write` scaffold loops
//! (used by `ws kb init` and future `ws init` scaffolding).
//!
//! This is mechanism-only: no Tier-1/2/3 asset subtrees live here yet.
//! Subsequent features drop their assets under this directory and build on
//! the macro.

use include_dir::{include_dir, Dir};

/// The canonical embedded-asset tree, rooted at `crates/ws-cli/assets/`.
///
/// Resolved via `$CARGO_MANIFEST_DIR` so it points at `ws-cli`'s own `assets/`
/// directory regardless of the working directory at build time.
///
/// Mechanism only (restructure ADR Q3). The first production consumer is the
/// `ws kb` epic's scaffolder; until it lands the symbol has no non-test caller,
/// which is expected for this deliberately-standalone wiring ticket.
#[allow(dead_code)]
pub static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke check: the embedded asset tree is non-empty and carries the
    /// `.gitkeep` sentinel — i.e. assets are embedded at build time and the
    /// `Dir` is iterable for future scaffold loops.
    #[test]
    fn assets_are_embedded_at_build_time() {
        assert!(
            !ASSETS.entries().is_empty(),
            "embedded asset tree is empty — assets/ must contain at least the .gitkeep sentinel"
        );
        assert!(
            ASSETS.get_file(".gitkeep").is_some(),
            "sentinel .gitkeep missing from embedded assets"
        );
    }
}
