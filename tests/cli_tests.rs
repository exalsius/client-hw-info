use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

#[test]
fn no_error() {
    let mut cmd = Command::cargo_bin("client-hw-info").unwrap();
    cmd.arg("--skip-heartbeat").assert().success();
    cmd.arg("--skip-heartbeat").assert().stderr(contains("ERROR").not());
    cmd.arg("--skip-heartbeat").assert().stderr(contains("Hardware collected"));
}