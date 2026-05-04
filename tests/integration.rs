use std::io::Write;
use std::process::{Command, Stdio};

fn spr() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spr"))
}

// ── Error-path tests (exit before TUI init — safe in any environment) ─────────

#[test]
fn no_args_exits_with_error() {
    // Explicitly close stdin so the process always sees an empty pipe,
    // giving deterministic behavior regardless of the test runner's environment.
    let mut child = spr()
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdin.take()); // immediate EOF
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty(), "expected an error message on stderr");
}

#[test]
fn text_arg_whitespace_only_exits_with_error() {
    let output = spr().args(["--text", "   \t\n"]).output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn file_arg_nonexistent_exits_with_error() {
    let output = spr()
        .args(["--file", "/nonexistent/path/spr_test_missing.txt"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn file_arg_directory_exits_with_error() {
    let output = spr().args(["--file", "/tmp"]).output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn stdin_pipe_empty_content_exits_with_error() {
    let mut child = spr()
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn stdin_pipe_whitespace_only_exits_with_error() {
    let mut child = spr()
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"   \n\t  ").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
}

// ── Happy-path tests (require a real TTY — run with `cargo test -- --ignored`) ─

/// Reads a single word at maximum WPM via --text.
/// Requires a controlling terminal (skipped in CI).
#[test]
#[ignore]
fn text_arg_single_word_exits_successfully() {
    let output = spr()
        .args(["--text", "hello", "--wpm", "999999"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Reads multiple words at maximum WPM via --text.
#[test]
#[ignore]
fn text_arg_multiple_words_exits_successfully() {
    let output = spr()
        .args(["--text", "one two three four five", "--wpm", "999999"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Reads a temp file at maximum WPM via --file.
#[test]
#[ignore]
fn file_arg_reads_content_and_exits_successfully() {
    let tmp = std::env::temp_dir().join("spr_integration_file_test.txt");
    std::fs::write(&tmp, "integration test content here").unwrap();
    let output = spr()
        .args(["--file", tmp.to_str().unwrap(), "--wpm", "999999"])
        .output()
        .unwrap();
    std::fs::remove_file(&tmp).ok();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Pipes content via stdin. Requires /dev/tty for keyboard event redirection.
#[test]
#[ignore]
fn stdin_pipe_with_content_exits_successfully() {
    if !std::path::Path::new("/dev/tty").exists() {
        return;
    }
    let mut child = spr()
        .args(["--wpm", "999999"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"hello from a pipe")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Pipes multi-line content via stdin.
#[test]
#[ignore]
fn stdin_pipe_multiline_exits_successfully() {
    if !std::path::Path::new("/dev/tty").exists() {
        return;
    }
    let content = b"first line of text\nsecond line of text\nthird line";
    let mut child = spr()
        .args(["--wpm", "999999"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(content).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
