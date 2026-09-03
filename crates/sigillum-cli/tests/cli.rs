use std::process::Command;

#[test]
fn version_reports_workspace_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_sigillum"))
        .arg("--version")
        .output()
        .expect("sigillum should start");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Sigillum 0.0.1\n");
}

#[test]
fn unknown_command_uses_usage_error_exit_code() {
    let status = Command::new(env!("CARGO_BIN_EXE_sigillum"))
        .arg("unknown")
        .status()
        .expect("sigillum should start");

    assert_eq!(status.code(), Some(2));
}

