use assert_cmd::Command;

#[test]
fn fails_on_unknown_arg() {
    let mut cmd = Command::cargo_bin("rtnt").unwrap();
    cmd.arg("--unknown").assert().failure();
}
