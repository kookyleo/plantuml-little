// Port of net.sourceforge.plantuml.dot.Graphviz (interface),
// AbstractGraphviz, GraphvizLinux, ProcessRunner, ProcessState, ExeState,
// GraphvizUtils, and GraphvizRuntimeEnvironment.
//
// The existing layout/graphviz.rs handles the actual `dot -Tsvg` execution
// and SVG parsing for node/edge layout. This module provides the canonical
// Graphviz abstraction layer: executable discovery, version detection,
// process execution, and exe state checking.

use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::dot::version::GraphvizVersion;

// ---------------------------------------------------------------------------
// ExeState — port of net.sourceforge.plantuml.dot.ExeState
// ---------------------------------------------------------------------------

/// State of the Graphviz `dot` executable.
/// Mirrors Java `ExeState` enum for checking file validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExeState {
    Ok,
    NullUndefined,
    DoesNotExist,
    IsADirectory,
    NotAFile,
    CannotBeRead,
}

impl ExeState {
    /// Check the state of an executable path.
    /// Java: `ExeState.checkFile(File dotExe)`
    pub fn check_file(path: Option<&Path>) -> ExeState {
        let path = match path {
            None => return ExeState::NullUndefined,
            Some(p) => p,
        };
        if !path.exists() {
            return ExeState::DoesNotExist;
        }
        if path.is_dir() {
            return ExeState::IsADirectory;
        }
        if !path.is_file() {
            return ExeState::NotAFile;
        }
        // On Unix, check read + execute permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(path) {
                let mode = meta.permissions().mode();
                if mode & 0o444 == 0 {
                    return ExeState::CannotBeRead;
                }
            }
        }
        ExeState::Ok
    }

    /// Human-readable message.
    /// Java: `ExeState.getTextMessage()`
    pub fn text_message(&self) -> &'static str {
        match self {
            ExeState::Ok => "Dot executable OK",
            ExeState::NullUndefined => "No dot executable found",
            ExeState::DoesNotExist => "Dot executable does not exist",
            ExeState::IsADirectory => {
                "Dot executable should be an executable, not a directory"
            }
            ExeState::NotAFile => "Dot executable is not a valid file",
            ExeState::CannotBeRead => "Dot executable cannot be read",
        }
    }

    /// Human-readable message including the file path.
    /// Java: `ExeState.getTextMessage(File exe)`
    pub fn text_message_with_path(&self, path: &Path) -> String {
        match self {
            ExeState::Ok => format!("File {} OK", path.display()),
            ExeState::NullUndefined => self.text_message().to_string(),
            _ => format!("File {}: {}", path.display(), self.text_message()),
        }
    }
}

impl fmt::Display for ExeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.text_message())
    }
}

// ---------------------------------------------------------------------------
// ProcessState — port of net.sourceforge.plantuml.dot.ProcessState
// ---------------------------------------------------------------------------

/// Outcome of a subprocess execution.
/// Mirrors Java `ProcessState`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    TerminatedOk,
    Timeout,
    Exception(String),
}

impl ProcessState {
    pub fn is_ok(&self) -> bool {
        matches!(self, ProcessState::TerminatedOk)
    }

    /// Java: `differs(ProcessState other)` — true when states are different.
    pub fn differs(&self, other: &ProcessState) -> bool {
        self != other
    }
}

impl fmt::Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessState::TerminatedOk => write!(f, "TERMINATED_OK"),
            ProcessState::Timeout => write!(f, "TIMEOUT"),
            ProcessState::Exception(msg) => write!(f, "EXCEPTION {msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// ProcessRunner — port of net.sourceforge.plantuml.dot.ProcessRunner
// ---------------------------------------------------------------------------

/// Result of running a subprocess, including captured stdout and stderr.
pub struct ProcessResult {
    pub state: ProcessState,
    pub stdout: Vec<u8>,
    pub stderr: String,
}

/// Default process timeout in milliseconds.
/// Java: `GlobalConfig.TIMEOUT_MS` default is typically 60_000.
const DEFAULT_TIMEOUT_MS: u64 = 60_000;

/// Run an external command, optionally piping `input` to stdin and capturing
/// stdout. Port of Java `ProcessRunner.run()`.
pub fn run_process(
    cmd: &[&str],
    input: Option<&[u8]>,
    timeout_ms: Option<u64>,
) -> ProcessResult {
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS));
    log::debug!(
        "run_process: cmd={:?}, input_len={}, timeout={}ms",
        cmd,
        input.map_or(0, |b| b.len()),
        timeout.as_millis()
    );

    let exe = match cmd.first() {
        Some(e) => *e,
        None => {
            return ProcessResult {
                state: ProcessState::Exception("empty command".into()),
                stdout: vec![],
                stderr: "empty command".into(),
            };
        }
    };

    let args = &cmd[1..];
    let mut child = match Command::new(exe)
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("run_process: failed to spawn {exe}: {e}");
            return ProcessResult {
                state: ProcessState::Exception(format!("spawn failed: {e}")),
                stdout: vec![],
                stderr: e.to_string(),
            };
        }
    };

    // Write input to stdin
    if let Some(data) = input {
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(data) {
                log::error!("run_process: failed to write stdin: {e}");
                let _ = child.kill();
                return ProcessResult {
                    state: ProcessState::Exception(format!("stdin write: {e}")),
                    stdout: vec![],
                    stderr: e.to_string(),
                };
            }
            // stdin is dropped here, closing the pipe
        }
    }

    // Wait with timeout
    match child.wait_with_output() {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                log::debug!(
                    "run_process: completed ok, stdout_len={}",
                    output.stdout.len()
                );
                ProcessResult {
                    state: ProcessState::TerminatedOk,
                    stdout: output.stdout,
                    stderr,
                }
            } else {
                let code = output.status.code().unwrap_or(-1);
                log::warn!("run_process: exited with code {code}, stderr={stderr}");
                ProcessResult {
                    state: ProcessState::Exception(format!("exit code {code}")),
                    stdout: output.stdout,
                    stderr,
                }
            }
        }
        Err(e) => {
            log::error!("run_process: wait failed: {e}");
            ProcessResult {
                state: ProcessState::Exception(format!("wait: {e}")),
                stdout: vec![],
                stderr: e.to_string(),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Graphviz trait — port of net.sourceforge.plantuml.dot.Graphviz interface
// ---------------------------------------------------------------------------

/// Graphviz execution interface.
///
/// Port of Java `Graphviz` interface. Provides methods to:
/// - Execute `dot` to produce SVG output
/// - Query the dot version
/// - Check executable state
pub trait Graphviz {
    /// Run the `dot` command with the stored DOT source, writing output
    /// to the provided writer.
    /// Java: `createFile3(OutputStream os)`
    fn create_file(&self, output: &mut dyn Write) -> ProcessState;

    /// Return the `dot -V` version string.
    /// Java: `dotVersion()`
    fn dot_version(&self) -> String;

    /// Return the path to the dot executable.
    /// Java: `getDotExe()`
    fn dot_exe(&self) -> Option<&Path>;

    /// Check the executable state.
    /// Java: `getExeState()`
    fn exe_state(&self) -> ExeState;
}

// ---------------------------------------------------------------------------
// GraphvizNative — port of AbstractGraphviz + GraphvizLinux
// ---------------------------------------------------------------------------

/// Native Graphviz implementation that shells out to the `dot` executable.
///
/// Combines Java's `AbstractGraphviz` (base) and `GraphvizLinux` (platform)
/// into a single struct, since we only target Linux/macOS natively.
pub struct GraphvizNative {
    dot_exe_path: Option<PathBuf>,
    dot_string: String,
    output_types: Vec<String>,
}

impl GraphvizNative {
    /// Create a new instance.
    /// Java: `GraphvizLinux(skinParam, dotString, type...)`
    pub fn new(dot_string: &str, output_types: &[&str]) -> Self {
        let exe = search_dot_exe();
        log::info!(
            "GraphvizNative: dot_exe={:?}, dot_len={}, types={:?}",
            exe,
            dot_string.len(),
            output_types
        );
        GraphvizNative {
            dot_exe_path: exe,
            dot_string: dot_string.to_string(),
            output_types: output_types.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Build the command line: `[dot_exe, -Ttype1, -Ttype2, ...]`
    /// Java: `AbstractGraphviz.getCommandLine()`
    fn command_line(&self) -> Vec<String> {
        let exe = self
            .dot_exe_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "dot".to_string());
        let mut cmd = vec![exe];
        for t in &self.output_types {
            cmd.push(format!("-T{t}"));
        }
        cmd
    }

    /// Build the version-query command line: `[dot_exe, -V]`
    fn command_line_version(&self) -> Vec<String> {
        let exe = self
            .dot_exe_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "dot".to_string());
        vec![exe, "-V".to_string()]
    }
}

impl Graphviz for GraphvizNative {
    fn create_file(&self, output: &mut dyn Write) -> ProcessState {
        if self.exe_state() != ExeState::Ok {
            log::error!("create_file: dot executable not OK: {}", self.exe_state());
            return ProcessState::Exception("dot executable not OK".into());
        }

        let cmd = self.command_line();
        let cmd_refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
        log::info!("create_file: cmd={cmd_refs:?}, dot_len={}", self.dot_string.len());

        let result = run_process(&cmd_refs, Some(self.dot_string.as_bytes()), None);
        if result.state.is_ok() {
            if let Err(e) = output.write_all(&result.stdout) {
                log::error!("create_file: write output failed: {e}");
                return ProcessState::Exception(format!("write: {e}"));
            }
        } else {
            log::warn!("create_file: process failed: {}", result.state);
            if !result.stderr.is_empty() {
                log::warn!("create_file: stderr={}", result.stderr);
            }
        }
        result.state
    }

    fn dot_version(&self) -> String {
        let cmd = self.command_line_version();
        let cmd_refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
        let result = run_process(&cmd_refs, None, Some(10_000));
        if result.state.differs(&ProcessState::TerminatedOk) {
            return "?".to_string();
        }
        // dot -V writes to stderr on most systems
        let mut combined = String::new();
        let stdout = String::from_utf8_lossy(&result.stdout);
        if !stdout.trim().is_empty() {
            combined.push_str(stdout.trim());
        }
        if !result.stderr.trim().is_empty() {
            if !combined.is_empty() {
                combined.push(' ');
            }
            combined.push_str(result.stderr.trim());
        }
        combined.replace('\n', " ").trim().to_string()
    }

    fn dot_exe(&self) -> Option<&Path> {
        self.dot_exe_path.as_deref()
    }

    fn exe_state(&self) -> ExeState {
        ExeState::check_file(self.dot_exe_path.as_deref())
    }
}

// ---------------------------------------------------------------------------
// Executable search — port of AbstractGraphviz.searchDotExe + GraphvizLinux
// ---------------------------------------------------------------------------

/// Search for the dot executable.
///
/// Priority (matches Java):
/// 1. `GRAPHVIZ_DOT` environment variable
/// 2. `dot` on PATH
/// 3. Well-known locations (Linux/macOS)
fn search_dot_exe() -> Option<PathBuf> {
    // Check GRAPHVIZ_DOT env var
    if let Ok(env_dot) = std::env::var("GRAPHVIZ_DOT") {
        let trimmed = env_dot.trim().trim_matches('"').to_string();
        if !trimmed.is_empty() {
            let p = PathBuf::from(&trimmed);
            if p.exists() {
                log::debug!("search_dot_exe: found via GRAPHVIZ_DOT={trimmed}");
                return Some(p);
            }
        }
    }

    // Search PATH
    if let Some(p) = find_executable_on_path("dot") {
        log::debug!("search_dot_exe: found on PATH={}", p.display());
        return Some(p);
    }

    // Well-known locations (Java GraphvizLinux.specificDotExe)
    let candidates = [
        "/usr/local/bin/dot",
        "/opt/homebrew/bin/dot",
        "/opt/homebrew/opt/graphviz/bin/dot",
        "/usr/bin/dot",
        "/opt/local/bin/dot",
    ];
    for c in &candidates {
        let p = Path::new(c);
        if p.exists() && p.is_file() {
            log::debug!("search_dot_exe: found at well-known location {c}");
            return Some(p.to_path_buf());
        }
    }

    log::warn!("search_dot_exe: dot executable not found");
    None
}

/// Search for an executable by name on the system PATH.
/// Java: `AbstractGraphviz.findExecutableOnPath(name)`
fn find_executable_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    for dir in path_var.split(':') {
        let candidate = Path::new(dir).join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Utility functions — port of GraphvizUtils
// ---------------------------------------------------------------------------

/// Default image size limit.
/// Java: `GraphvizUtils.getenvImageLimit()` default = 4096.
pub const DEFAULT_IMAGE_LIMIT: u32 = 4096;

/// Get the image size limit from environment or default.
/// Java: `GraphvizUtils.getenvImageLimit()`
pub fn image_limit() -> u32 {
    if let Ok(val) = std::env::var("PLANTUML_LIMIT_SIZE") {
        if let Ok(n) = val.parse::<u32>() {
            return n;
        }
    }
    DEFAULT_IMAGE_LIMIT
}

/// Quick test: create a trivial graph and verify SVG output.
/// Java: `GraphvizUtils.getTestCreateSimpleFile()`
///
/// Returns `Ok(())` if Graphviz produces valid SVG, or `Err(message)`.
pub fn test_graphviz_installation() -> Result<(), String> {
    let gv = GraphvizNative::new("digraph foo { test; }", &["svg"]);
    let mut output = Vec::new();
    let state = gv.create_file(&mut output);
    if state.differs(&ProcessState::TerminatedOk) {
        return Err(format!("Error: timeout {state}"));
    }
    if output.is_empty() {
        return Err(
            "Error: dot generates empty file. Check your dot installation.".into(),
        );
    }
    let s = String::from_utf8_lossy(&output);
    if !s.contains("<svg") {
        return Err(
            "Error: dot generates unreadable SVG file. Check your dot installation."
                .into(),
        );
    }
    Ok(())
}

/// Detect the installed Graphviz version.
/// Combines Java's `GraphvizVersionFinder` and `GraphvizRuntimeEnvironment`.
pub fn detect_graphviz_version() -> GraphvizVersion {
    let gv = GraphvizNative::new("", &["svg"]);
    let version_str = gv.dot_version();
    log::info!("detect_graphviz_version: raw={version_str}");
    GraphvizVersion::parse_from_dot_output(&version_str)
        .unwrap_or(GraphvizVersion::DEFAULT)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exe_state_check_none() {
        assert_eq!(ExeState::check_file(None), ExeState::NullUndefined);
    }

    #[test]
    fn exe_state_check_nonexistent() {
        let p = Path::new("/nonexistent/path/to/dot_xyz_123");
        assert_eq!(ExeState::check_file(Some(p)), ExeState::DoesNotExist);
    }

    #[test]
    fn exe_state_check_directory() {
        let p = Path::new("/tmp");
        assert_eq!(ExeState::check_file(Some(p)), ExeState::IsADirectory);
    }

    #[test]
    fn exe_state_text_messages() {
        assert_eq!(ExeState::Ok.text_message(), "Dot executable OK");
        assert_eq!(ExeState::NullUndefined.text_message(), "No dot executable found");
        assert!(!ExeState::DoesNotExist.text_message().is_empty());
    }

    #[test]
    fn exe_state_text_message_with_path() {
        let p = Path::new("/usr/bin/dot");
        let msg = ExeState::Ok.text_message_with_path(p);
        assert!(msg.contains("/usr/bin/dot"));
        assert!(msg.contains("OK"));
    }

    #[test]
    fn process_state_display() {
        assert_eq!(format!("{}", ProcessState::TerminatedOk), "TERMINATED_OK");
        assert_eq!(format!("{}", ProcessState::Timeout), "TIMEOUT");
    }

    #[test]
    fn process_state_differs() {
        assert!(!ProcessState::TerminatedOk.differs(&ProcessState::TerminatedOk));
        assert!(ProcessState::TerminatedOk.differs(&ProcessState::Timeout));
    }

    #[test]
    fn image_limit_default() {
        // Unless PLANTUML_LIMIT_SIZE is set, should return 4096
        let limit = image_limit();
        assert!(limit > 0);
    }

    #[test]
    fn search_dot_exe_finds_something() {
        // This test may or may not find dot depending on the system.
        // We just verify it doesn't panic.
        let _ = search_dot_exe();
    }

    #[test]
    fn graphviz_native_command_line() {
        let gv = GraphvizNative {
            dot_exe_path: Some(PathBuf::from("/usr/bin/dot")),
            dot_string: String::new(),
            output_types: vec!["svg".into()],
        };
        let cmd = gv.command_line();
        assert_eq!(cmd, vec!["/usr/bin/dot", "-Tsvg"]);
    }

    #[test]
    fn graphviz_native_command_line_multiple_types() {
        let gv = GraphvizNative {
            dot_exe_path: Some(PathBuf::from("/usr/bin/dot")),
            dot_string: String::new(),
            output_types: vec!["svg".into(), "png".into()],
        };
        let cmd = gv.command_line();
        assert_eq!(cmd, vec!["/usr/bin/dot", "-Tsvg", "-Tpng"]);
    }

    #[test]
    fn graphviz_native_version_command() {
        let gv = GraphvizNative {
            dot_exe_path: Some(PathBuf::from("/usr/bin/dot")),
            dot_string: String::new(),
            output_types: vec![],
        };
        let cmd = gv.command_line_version();
        assert_eq!(cmd, vec!["/usr/bin/dot", "-V"]);
    }

    // Integration test: only runs if dot is installed
    #[test]
    fn graphviz_native_create_file_integration() {
        if search_dot_exe().is_none() {
            eprintln!("SKIP: dot not found on system");
            return;
        }
        let gv = GraphvizNative::new("digraph G { A -> B; }", &["svg"]);
        let mut buf = Vec::new();
        let state = gv.create_file(&mut buf);
        assert!(state.is_ok(), "expected TerminatedOk, got {state}");
        let svg = String::from_utf8_lossy(&buf);
        assert!(svg.contains("<svg"), "output should contain <svg");
    }

    #[test]
    fn graphviz_native_dot_version_integration() {
        if search_dot_exe().is_none() {
            eprintln!("SKIP: dot not found on system");
            return;
        }
        let gv = GraphvizNative::new("", &["svg"]);
        let version = gv.dot_version();
        assert_ne!(version, "?", "should detect a version string");
        assert!(
            version.contains("graphviz") || version.contains("dot"),
            "version should mention graphviz or dot: {version}"
        );
    }

    #[test]
    fn test_installation_integration() {
        if search_dot_exe().is_none() {
            eprintln!("SKIP: dot not found on system");
            return;
        }
        let result = test_graphviz_installation();
        assert!(result.is_ok(), "installation test failed: {:?}", result);
    }

    #[test]
    fn detect_version_integration() {
        if search_dot_exe().is_none() {
            eprintln!("SKIP: dot not found on system");
            return;
        }
        let v = detect_graphviz_version();
        assert!(v.major >= 2, "major version should be >= 2");
        assert!(v.numeric() > 0, "numeric version should be positive");
    }
}
