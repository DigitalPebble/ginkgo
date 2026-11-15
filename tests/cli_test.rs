// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

fn run(repository: &str, dir: &std::path::Path) -> (bool, String) {
    let input = dir.join("bill.json");
    let out = dir.join("out.json");
    let item = |repo: &str| {
        format!(
            r#"{{"date":"2025-01-01","product":"actions","sku":"actions_linux","quantity":10.0,
            "unitType":"minutes","pricePerUnit":0.008,"grossAmount":0.08,"discountAmount":0.08,
            "netAmount":0.0,"organizationName":"DigitalPebble","repositoryName":"{repo}"}}"#
        )
    };
    std::fs::write(
        &input,
        format!(
            r#"{{"usageItems":[{},{}]}}"#,
            item("public-repo"),
            item("secret-repo")
        ),
    )
    .unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_ginkgo"))
        .args([
            "--file",
            input.to_str().unwrap(),
            "-O",
            out.to_str().unwrap(),
        ])
        .args(["--repository", repository])
        .env_remove("INPUT_REPOSITORY")
        .env_remove("GITHUB_STEP_SUMMARY")
        .env_remove("GITHUB_OUTPUT")
        .status()
        .unwrap();
    let report = std::fs::read_to_string(&out).unwrap_or_default();
    (status.success(), report)
}

#[test]
fn report_only_contains_the_selected_repository() {
    let dir = std::env::temp_dir().join(format!("ginkgo-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (ok, report) = run("DigitalPebble/public-repo", &dir);
    assert!(ok);
    assert!(report.contains("public-repo"));
    assert!(!report.contains("secret-repo"));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn invalid_repository_fails() {
    let dir = std::env::temp_dir().join(format!("ginkgo-cli-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (ok, report) = run("not-a-repo", &dir);
    assert!(!ok);
    assert!(report.is_empty());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn badge_is_written_when_requested() {
    let dir = std::env::temp_dir().join(format!("ginkgo-cli-badge-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let badge = dir.join("sub").join("badge.json");
    let input = dir.join("bill.json");
    std::fs::write(&input, r#"{"usageItems":[]}"#).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_ginkgo"))
        .args(["--file", input.to_str().unwrap()])
        .args(["-O", dir.join("out.json").to_str().unwrap()])
        .args(["--badge", badge.to_str().unwrap()])
        .env_remove("GITHUB_STEP_SUMMARY")
        .env_remove("GITHUB_OUTPUT")
        .status()
        .unwrap();
    assert!(status.success());

    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&badge).unwrap()).unwrap();
    assert_eq!(v["schemaVersion"], 1);
    assert_eq!(v["message"], "0.0 gCO2e");
    assert_eq!(v["color"], "ffd700"); // data read from a file is billing data: gold
    std::fs::remove_dir_all(&dir).ok();
}
