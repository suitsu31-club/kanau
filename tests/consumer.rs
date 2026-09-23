//! Enabling `kanau/message` plus a backend feature must be enough to use that
//! backend's derives: the consumer needs no direct dependency on the codec crate.
//!
//! trybuild cannot check this, because it copies every dependency of `kanau` into
//! its fixture crates, so the codecs always resolve at the fixture's crate root.
//! Each fixture under `tests/consumer/` is instead built and run as a standalone
//! crate whose only dependencies are the ones declared here.

#![allow(dead_code)]

use std::fs;
use std::path::Path;
use std::process::Command;

fn run_consumer(fixture: &str, kanau_feature: &str, extra_dependencies: &str) {
    let kanau = Path::new(env!("CARGO_MANIFEST_DIR"));
    let consumers = Path::new(env!("CARGO_TARGET_TMPDIR")).join("consumer");
    let project = consumers.join(fixture);
    fs::create_dir_all(&project).unwrap();

    let manifest = format!(
        r#"[package]
name = "kanau-consumer-{fixture}"
version = "0.0.0"
edition = "2024"
publish = false

[[bin]]
name = "consumer"
path = '{main}'

[dependencies]
kanau = {{ path = '{kanau}', features = ["message", "{kanau_feature}"] }}
{extra_dependencies}

[workspace]
"#,
        main = kanau
            .join("tests/consumer")
            .join(format!("{fixture}.rs"))
            .display(),
        kanau = kanau.display(),
    );
    fs::write(project.join("Cargo.toml"), manifest).unwrap();
    // Pin the versions `kanau` itself is tested against.
    fs::copy(kanau.join("Cargo.lock"), project.join("Cargo.lock")).unwrap();

    let output = Command::new(env!("CARGO"))
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(project.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(consumers.join("target"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "consumer `{fixture}` failed:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
}

#[cfg(all(feature = "message", feature = "serde_json"))]
#[test]
fn serde_json_needs_no_direct_dependency() {
    run_consumer(
        "serde_json",
        "serde_json",
        r#"serde = { version = "1", features = ["derive"] }"#,
    );
}

#[cfg(all(feature = "message", feature = "bincode"))]
#[test]
fn bincode_needs_no_direct_dependency() {
    run_consumer("bincode", "bincode", "");
}

#[cfg(all(feature = "message", feature = "rkyv"))]
#[test]
fn rkyv_needs_no_direct_dependency() {
    run_consumer("rkyv", "rkyv", "");
}

#[cfg(all(feature = "message", feature = "musli-wire"))]
#[test]
fn musli_wire_needs_no_direct_dependency() {
    run_consumer("musli_wire", "musli-wire", "");
}

#[cfg(all(feature = "message", feature = "prost"))]
#[test]
fn prost_needs_no_direct_dependency() {
    run_consumer("prost", "prost", "");
}
