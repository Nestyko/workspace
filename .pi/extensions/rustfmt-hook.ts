/**
 * Rust Formatter Hook (post-edit)
 *
 * After the agent edits or writes a `.rs` file via the `edit` or `write`
 * tools, automatically run `rustfmt` on just that file, behind the scenes.
 *
 * This is the "formatting on edit" layer of the formatter story; the
 * pre-commit git hook (`.githooks/pre-commit`) is the commit-time layer.
 *
 * Scope is deliberately per-file: it never runs `cargo fmt --all`, so the
 * workspace scales without reformatting untouched code.
 */
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { execFile } from "node:child_process";
import { resolve, extname } from "node:path";
import { promisify } from "node:util";

const execFileP = promisify(execFile);

const RUSTFMT_TIMEOUT_MS = 15_000;

/** Tools that write file contents and carry a `path` argument. */
const MUTATING_TOOLS = new Set(["edit", "write"]);

function inlineRustFiles(input: unknown): string[] {
  if (!input || typeof input !== "object") return [];
  const obj = input as Record<string, unknown>;
  const pathVal = obj.path;
  if (typeof pathVal === "string" && extname(pathVal) === ".rs") {
    return [pathVal];
  }
  // Some tools may nest paths under edits[]; `edit` uses a top-level `path`,
  // but be defensive and never throw if the shape differs.
  return [];
}

export default function (pi: ExtensionAPI) {
  pi.on("tool_result", async (event, ctx) => {
    // Only react to successful write/edit operations on .rs files.
    if (!MUTATING_TOOLS.has(event.toolName)) return;
    if (event.isError) return;

    const targets = inlineRustFiles(event.input);
    if (targets.length === 0) return;

    const cwd = ctx.cwd;

    for (const rel of targets) {
      const abs = resolve(cwd, rel);
      // Skip anything outside the workspace (e.g. absolute paths elsewhere).
      if (!abs.startsWith(cwd)) continue;

      try {
        // Pin the edition to match the workspace (Cargo.toml sets
        // edition = "2021"). Without this rustfmt defaults to 2015 and
        // errors on `async fn`, `dyn Trait` shorthand, etc.
        await execFileP("rustfmt", ["--edition", "2021", abs], {
          cwd,
          timeout: RUSTFMT_TIMEOUT_MS,
          maxBuffer: 4 * 1024 * 1024,
        });
      } catch (err: unknown) {
        // Best-effort: never break the agent's edit flow over a formatting hiccup.
        const msg = err instanceof Error ? err.message : String(err);
        if (ctx.hasUI) {
          ctx.ui.setStatus(
            "rustfmt",
            `rustfmt failed on ${rel}: ${msg.split("\n")[0]}`,
          );
        }
      }
    }
  });
}
