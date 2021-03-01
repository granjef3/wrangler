use std::fs;
use std::process::Command;
use std::str;

use assert_cmd::prelude::*;
use wrangler::fixtures::{Fixture, WranglerToml};

fn build_creates_assets_with_arg(
    fixture: &Fixture,
    script_names: Vec<&str>,
    args: Vec<&str>,
) -> (String, String) {
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(fixture.get_path());
    build.arg("build");
    build.args(args);

    let output = build.output().expect("failed to execute process");
    assert!(output.status.success());

    for script_name in script_names {
        assert!(fixture.get_output_path().join(script_name).exists());
    }

    (
        str::from_utf8(&output.stdout).unwrap().to_string(),
        str::from_utf8(&output.stderr).unwrap().to_string(),
    )
}

fn build_creates_assets(fixture: &Fixture, script_names: Vec<&str>) -> (String, String) {
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(fixture.get_path());
    build.arg("build");

    let output = build.output().expect("failed to execute process");
    assert!(output.status.success());

    for script_name in script_names {
        assert!(fixture.get_output_path().join(script_name).exists());
    }

    (
        str::from_utf8(&output.stdout).unwrap().to_string(),
        str::from_utf8(&output.stderr).unwrap().to_string(),
    )
}

fn build_fails_with(fixture: &Fixture, expected_message: &str) {
    let mut build = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    build.current_dir(fixture.get_path());
    build.arg("build");

    let output = build.output().expect("failed to execute process");
    assert!(!output.status.success());
    assert!(
        str::from_utf8(&output.stderr)
            .unwrap()
            .contains(expected_message),
        format!(
            "expected {:?} not found, given: {:?}",
            expected_message,
            str::from_utf8(&output.stderr)
        )
    );
}
