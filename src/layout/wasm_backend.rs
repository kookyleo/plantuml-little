//! Deterministic Graphviz backend for plantuml-little's reference tests.
//!
//! Production code paths go through [`super::graphviz::render_dot_to_svg`],
//! which defaults to the native `graphviz-anywhere` crate. When the env
//! var `PLANTUML_LITTLE_TEST_BACKEND=wasm` is set, `render_dot_to_svg`
//! delegates here instead: we spawn a long-lived Node.js child process
//! that loads `@kookyleo/graphviz-anywhere-web` (a wasm-compiled
//! Graphviz; version pinned in `tests/support/package.json`) and
//! streams DOT → SVG requests over stdin/stdout.
//!
//! Why Node + wasm? The same viz.wasm bytes ship to every OS and every
//! CI runner; V8's wasm runtime is spec-deterministic for Graphviz's
//! float math, so every reference test produces the exact same SVG
//! bytes regardless of where it runs. The native backend still works
//! for production users (they accept sub-pixel drift), but the CI
//! reference-test gate uses this wasm backend to guarantee
//! byte-identical SVGs across runners.
//!
//! See `tests/support/viz_wasm_runner.mjs` for the Node-side script
//! and framing protocol.

use crate::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Mutex, OnceLock};

/// Env var that opts the test harness into the wasm backend. Any value
/// other than the empty string activates it; conventional values are
/// `wasm` or `1`.
pub const BACKEND_ENV_VAR: &str = "PLANTUML_LITTLE_TEST_BACKEND";

/// Returns true iff the env var selects the wasm backend.
pub fn wasm_backend_selected() -> bool {
    match std::env::var(BACKEND_ENV_VAR) {
        Ok(v) => {
            let v = v.trim();
            !v.is_empty() && !v.eq_ignore_ascii_case("native") && !v.eq_ignore_ascii_case("off")
        }
        Err(_) => false,
    }
}

/// Long-lived Node subprocess running `tests/support/viz_wasm_runner.mjs`.
struct WasmRunner {
    // Child is kept alive for the lifetime of the static singleton.
    // Dropped implicitly on process exit.
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl WasmRunner {
    fn spawn() -> Result<Self, Error> {
        let runner_script = locate_runner_script()?;
        let working_dir = runner_script
            .parent()
            .ok_or_else(|| Error::Layout("viz_wasm_runner has no parent dir".to_string()))?
            .to_path_buf();

        // NODE_NO_WARNINGS silences the experimental-warning chatter that
        // some Node builds emit for wasm ESM — those warnings go to
        // stderr which we leave inheriting from the parent anyway, so
        // they're harmless, but silencing them keeps CI logs clean.
        let mut cmd = Command::new("node");
        cmd.arg(&runner_script)
            .current_dir(&working_dir)
            .env("NODE_NO_WARNINGS", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = cmd.spawn().map_err(|e| {
            Error::Layout(format!(
                "failed to spawn node for wasm Graphviz backend (is Node.js installed?): {e}. \
                 Script path: {}",
                runner_script.display()
            ))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::Layout("node child has no stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::Layout("node child has no stdout".to_string()))?;
        let mut stdout = BufReader::new(stdout);

        // Handshake: the runner prints `READY <version>\n` once wasm is loaded.
        let mut ready = String::new();
        stdout.read_line(&mut ready).map_err(|e| {
            Error::Layout(format!(
                "wasm Graphviz backend: failed to read READY line from node: {e}. \
                 Did `npm install` run in tests/support/?"
            ))
        })?;
        if !ready.starts_with("READY ") {
            return Err(Error::Layout(format!(
                "wasm Graphviz backend: unexpected handshake from node (expected 'READY <version>'): {ready:?}"
            )));
        }

        Ok(WasmRunner {
            _child: child,
            stdin,
            stdout,
        })
    }

    fn render(&mut self, dot_src: &str) -> Result<String, Error> {
        let bytes = dot_src.as_bytes();
        // Request framing: length\n, dot bytes, \n.
        writeln!(self.stdin, "{}", bytes.len())
            .and_then(|()| self.stdin.write_all(bytes))
            .and_then(|()| self.stdin.write_all(b"\n"))
            .and_then(|()| self.stdin.flush())
            .map_err(|e| {
                Error::Layout(format!(
                    "wasm Graphviz backend: failed to write request to node: {e}"
                ))
            })?;

        // Response framing: status\n, length\n, payload, \n.
        let mut status = String::new();
        self.stdout.read_line(&mut status).map_err(|e| {
            Error::Layout(format!(
                "wasm Graphviz backend: failed to read status line: {e}"
            ))
        })?;
        let status = status.trim_end_matches(['\r', '\n']);

        let mut len_line = String::new();
        self.stdout.read_line(&mut len_line).map_err(|e| {
            Error::Layout(format!(
                "wasm Graphviz backend: failed to read length line: {e}"
            ))
        })?;
        let len: usize = len_line
            .trim_end_matches(['\r', '\n'])
            .parse()
            .map_err(|e| {
                Error::Layout(format!(
                    "wasm Graphviz backend: bad length line {len_line:?}: {e}"
                ))
            })?;

        let mut payload = vec![0u8; len];
        self.stdout.read_exact(&mut payload).map_err(|e| {
            Error::Layout(format!(
                "wasm Graphviz backend: failed to read {len}-byte payload: {e}"
            ))
        })?;
        // Consume trailing '\n' separator.
        let mut trailing = [0u8; 1];
        self.stdout.read_exact(&mut trailing).map_err(|e| {
            Error::Layout(format!(
                "wasm Graphviz backend: failed to read trailing newline: {e}"
            ))
        })?;

        let payload_str = String::from_utf8(payload)
            .map_err(|e| Error::Layout(format!("wasm Graphviz backend: non-UTF-8 payload: {e}")))?;

        match status {
            "OK" => Ok(payload_str),
            "ERR" => Err(Error::Layout(format!(
                "wasm Graphviz backend: render failed: {payload_str}"
            ))),
            other => Err(Error::Layout(format!(
                "wasm Graphviz backend: unexpected status {other:?} (payload: {payload_str})"
            ))),
        }
    }
}

/// Render a DOT source string to SVG using the Node+wasm backend.
///
/// Spawns a single Node subprocess on first call and reuses it for
/// subsequent renders. A process-wide mutex serialises requests.
pub fn render_dot_to_svg(dot_src: &str) -> Result<String, Error> {
    static RUNNER: OnceLock<Mutex<WasmRunner>> = OnceLock::new();
    let mutex = match RUNNER.get() {
        Some(m) => m,
        None => {
            let runner = WasmRunner::spawn()?;
            // Safe: init-or-get semantics via set().
            let _ = RUNNER.set(Mutex::new(runner));
            RUNNER
                .get()
                .expect("RUNNER initialised above or by another thread")
        }
    };
    let mut guard = mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.render(dot_src)
}

/// Find `tests/support/viz_wasm_runner.mjs` relative to the crate root.
///
/// In tests, `CARGO_MANIFEST_DIR` points at the crate root, so we
/// anchor there. As a fallback, walk up from `current_dir()` looking
/// for `tests/support/viz_wasm_runner.mjs` — useful when tests are run
/// from a sub-directory.
fn locate_runner_script() -> Result<PathBuf, Error> {
    const REL: &str = "tests/support/viz_wasm_runner.mjs";

    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        let p = Path::new(manifest_dir).join(REL);
        if p.exists() {
            return Ok(p);
        }
    }

    if let Ok(mut cur) = std::env::current_dir() {
        loop {
            let p = cur.join(REL);
            if p.exists() {
                return Ok(p);
            }
            if !cur.pop() {
                break;
            }
        }
    }

    Err(Error::Layout(format!(
        "wasm Graphviz backend: could not locate {REL} (tried CARGO_MANIFEST_DIR and parent dirs). \
         This backend is only intended for tests — set PLANTUML_LITTLE_TEST_BACKEND=native to \
         disable it or ensure the script ships with the test harness."
    )))
}
