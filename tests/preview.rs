use wrangler::fixtures::{Fixture, WranglerToml};

use std::collections::HashMap;
use std::env;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn it_can_preview_js_project() {
    let fixture = Fixture::new();
    fixture.create_file(
        "index.js",
        r#"
        addEventListener('fetch', event => {
            event.respondWith(handleRequest(event.request))
        })

        /**
        * Fetch and log a request
        * @param {Request} request
        */
        async function handleRequest(request) {
            return new Response('Hello worker!', { status: 200 })
        }
    "#,
    );
    fixture.create_default_package_json();

    let wrangler_toml = WranglerToml::javascript("test-preview-javascript");
    fixture.create_wrangler_toml(wrangler_toml);

    preview_succeeds(&fixture);
}

#[test]
fn it_can_preview_using_url_flag() {
    let fixture = Fixture::new();
    fixture.create_file(
        "index.js",
        r#"
        addEventListener('fetch', event => {
            event.respondWith(handleRequest(event.request))
        })

        async function handleRequest(request) {
            return new Response(request.url, { status: 200 })
        }
    "#,
    );
    fixture.create_default_package_json();

    let wrangler_toml = WranglerToml::javascript("test-preview-javascript");
    fixture.create_wrangler_toml(wrangler_toml);

    // URLs should match as expected
    preview_matches_url(&fixture, "https://example.com/a", "https://example.com/a");

    // URLs should not match as expected
    preview_not_matches_url(&fixture, "https://example.com/a", "https://example.com/b");
}

#[test]
fn it_previews_with_config_text() {
    let fixture = Fixture::new();
    fixture.create_file(
        "index.js",
        r#"
        addEventListener('fetch', event => {
            event.respondWith(handleRequest(event.request))
        })

        async function handleRequest(request) {
            return new Response(CONFIG_TEST)
        }
    "#,
    );
    fixture.create_default_package_json();

    let test_value: &'static str = "sdhftiuyrtdhfjgpoopuyrdfjgkyitudrhf";

    let mut wrangler_toml = WranglerToml::javascript("test-preview-with-config");
    let mut config: HashMap<&'static str, &'static str> = HashMap::new();
    config.insert("CONFIG_TEST", test_value);
    wrangler_toml.vars = Some(config);
    fixture.create_wrangler_toml(wrangler_toml);

    preview_succeeds_with(&fixture, None, test_value);
}

// TODO: test custom build preview

#[test]
fn it_previews_with_text_blob() {
    let fixture = Fixture::new();
    fixture.create_file(
        "index.js",
        r#"
        addEventListener('fetch', event => {
            event.respondWith(handleRequest(event.request))
        })

        async function handleRequest(request) {
            return new Response(BLOB)
        }
    "#,
    );
    fixture.create_default_package_json();

    let test_value: &'static str = "sdhftiuyrtdhfjgpoopuyrdfjgkyitudrhf";
    fixture.create_file("blob.txt", test_value);

    let mut wrangler_toml = WranglerToml::javascript("test-preview-with-config");
    let mut blobs: HashMap<&'static str, &'static str> = HashMap::new();
    blobs.insert("BLOB", "blob.txt");
    wrangler_toml.text_blobs = Some(blobs);
    fixture.create_wrangler_toml(wrangler_toml);

    preview_succeeds_with(&fixture, None, test_value);
}

fn preview_succeeds_with(fixture: &Fixture, env: Option<&str>, expected: &str) {
    env::remove_var("CF_ACCOUNT_ID");
    env::remove_var("CF_ZONE_ID");
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(fixture.get_path());
    preview.arg("preview").arg("--headless");
    if let Some(env) = env {
        preview.arg("--env").arg(env);
    }
    preview
        .assert()
        .stdout(predicates::str::contains(expected))
        .success();
}

fn preview_succeeds(fixture: &Fixture) {
    env::remove_var("CF_ACCOUNT_ID");
    env::remove_var("CF_ZONE_ID");
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(fixture.get_path());
    preview.arg("preview").arg("--headless");
    preview.assert().success();
}

fn preview_matches_url(fixture: &Fixture, url: &str, expected: &str) {
    env::remove_var("CF_ACCOUNT_ID");
    env::remove_var("CF_ZONE_ID");
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(fixture.get_path());
    preview.arg("preview").arg("--headless");
    preview.arg("--url").arg(url);
    preview.assert().stdout(predicate::str::contains(expected));
}

fn preview_not_matches_url(fixture: &Fixture, url: &str, expected: &str) {
    env::remove_var("CF_ACCOUNT_ID");
    env::remove_var("CF_ZONE_ID");
    let mut preview = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    preview.current_dir(fixture.get_path());
    preview.arg("preview").arg("--headless");
    preview.arg("--url").arg(url);
    preview
        .assert()
        .stdout(predicate::str::contains(expected).not());
}
