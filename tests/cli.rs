//! Integration tests for the command line interface.
use assert_cmd::Command;

#[test]
fn runs_help() {
    let mut cmd = Command::cargo_bin("rtnt").unwrap();
    cmd.arg("--help").assert().success();
}

#[test]
fn runs_version() {
    let mut cmd = Command::cargo_bin("rtnt").unwrap();
    cmd.arg("-V").assert().success();
}

#[test]
fn runs_verbose() {
    let mut cmd = Command::cargo_bin("rtnt").unwrap();
    cmd.args(["-v", "--help"]).assert().success();
}
