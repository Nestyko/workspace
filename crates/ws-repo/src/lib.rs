//! `ws-repo` — repo-level deterministic AI commands for the repo-init healthcheck
//! surface: `repo.healthcheck`, `repo.run`, `repo.verify`, `repo.fix_loop.prompt`,
//! and `repo.understand.verify`.
//!
//! Invariant: `ws` is the deterministic oracle. These commands never fix, never
//! scaffold, never judge, and never own an LLM. They read, execute primitives,
//! emit specs, and validate writes. The harness fills gaps and drives fix-loops.

pub mod fix_loop;
pub mod healthcheck;
pub mod run;
pub mod understand;
pub mod verify;

pub use fix_loop::{RepoFixLoopPromptCommand, RepoFixLoopPromptInput, RepoFixLoopPromptOutput};
pub use healthcheck::{RepoHealthcheckCommand, RepoHealthcheckInput, RepoHealthcheckOutput};
pub use run::{RepoRunCommand, RepoRunInput, RepoRunOutput};
pub use understand::{
    RepoUnderstandVerifyCommand, RepoUnderstandVerifyInput, RepoUnderstandVerifyOutput,
};
pub use verify::{RepoVerifyCommand, RepoVerifyInput, RepoVerifyOutput};
