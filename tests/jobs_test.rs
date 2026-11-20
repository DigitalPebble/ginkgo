// SPDX-License-Identifier: Apache-2.0

use aspen::estimator::calculate_carbon_impact;
use aspen::jobs::{items_from_jobs, sku_for_labels};
use aspen::model::ActionsBill;

fn labels(l: &[&str]) -> Vec<String> {
    l.iter().map(|s| s.to_string()).collect()
}

const JOBS: &str = r#"{"total_count":5,"jobs":[
 {"status":"completed","conclusion":"success","started_at":"2025-03-01T10:00:00Z","completed_at":"2025-03-01T10:01:01Z","labels":["ubuntu-latest"]},
 {"status":"completed","conclusion":"success","started_at":"2025-03-01T10:00:00Z","completed_at":"2025-03-01T10:03:00Z","labels":["ubuntu-24.04-arm"]},
 {"status":"completed","conclusion":"skipped","started_at":"2025-03-01T10:00:00Z","completed_at":"2025-03-01T10:00:00Z","labels":["ubuntu-latest"]},
 {"status":"queued","conclusion":null,"started_at":null,"completed_at":null,"labels":["ubuntu-latest"]},
 {"status":"completed","conclusion":"success","started_at":"2025-03-01T10:00:00Z","completed_at":"2025-03-01T10:00:30Z","labels":["self-hosted","linux"]}
]}"#;

#[test]
fn maps_labels_to_billing_skus() {
    assert_eq!(sku_for_labels(&labels(&["ubuntu-latest"])), "Actions Linux");
    assert_eq!(sku_for_labels(&labels(&["ubuntu-22.04"])), "Actions Linux");
    assert_eq!(
        sku_for_labels(&labels(&["ubuntu-24.04-arm"])),
        "Actions Linux ARM"
    );
    assert_eq!(
        sku_for_labels(&labels(&["windows-latest"])),
        "Actions Windows"
    );
    assert_eq!(
        sku_for_labels(&labels(&["macos-14"])),
        "Actions macOS 3-core"
    );
    assert_eq!(
        sku_for_labels(&labels(&["macos-latest"])),
        "Actions macOS 3-core"
    );
    assert_eq!(
        sku_for_labels(&labels(&["macos-15-intel"])),
        "Actions macOS 4-core"
    );
    assert_eq!(
        sku_for_labels(&labels(&["macos-13"])),
        "Actions macOS 4-core"
    );
    // larger runners have other hardware and are not estimated
    assert_eq!(
        sku_for_labels(&labels(&["macos-15-xlarge"])),
        "macos-15-xlarge"
    );
    assert_eq!(
        sku_for_labels(&labels(&["macos-26-large"])),
        "macos-26-large"
    );
    assert_eq!(
        sku_for_labels(&labels(&["self-hosted", "linux"])),
        "Self-hosted"
    );
    assert_eq!(sku_for_labels(&labels(&["my-big-runner"])), "my-big-runner");
}

#[test]
fn rounds_up_minutes_and_ignores_jobs_that_did_not_run() {
    let items = items_from_jobs(JOBS, "DigitalPebble", "aspen").unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].sku, "Actions Linux");
    assert_eq!(items[0].quantity, 2.0); // 61s rounds up to 2 minutes
    assert_eq!(items[1].sku, "Actions Linux ARM");
    assert_eq!(items[1].quantity, 3.0);
    assert_eq!(items[2].sku, "Self-hosted");
    assert_eq!(items[2].quantity, 1.0);
    assert!(items
        .iter()
        .all(|i| i.organization_name == "DigitalPebble" && i.repository_name == "aspen"));
}

#[test]
fn items_feed_the_estimator() {
    let mut bill = ActionsBill {
        usage_items: items_from_jobs(JOBS, "o", "r").unwrap(),
    };
    calculate_carbon_impact(&mut bill);
    assert!(bill.usage_items[0].co2eq_g.unwrap() > 0.0);
    assert!(bill.usage_items[1].co2eq_g.unwrap() > 0.0);
    assert!(bill.usage_items[2].co2eq_g.is_none());
}

#[test]
fn empty_page_gives_no_items() {
    assert!(items_from_jobs(r#"{"jobs":[]}"#, "o", "r")
        .unwrap()
        .is_empty());
}

#[test]
fn summary_has_a_workflow_table_when_names_are_known() {
    let mut bill = ActionsBill {
        usage_items: items_from_jobs(JOBS, "DigitalPebble", "aspen").unwrap(),
    };
    assert!(!aspen::summary::to_markdown(&{
        let mut b = bill.clone();
        calculate_carbon_impact(&mut b);
        b
    })
    .contains("By workflow"));

    for item in &mut bill.usage_items {
        item.workflow_name = Some("CI".to_string());
        item.workflow_path = Some(".github/workflows/ci.yml".to_string());
    }
    calculate_carbon_impact(&mut bill);
    let md = aspen::summary::to_markdown(&bill);
    assert!(md.contains("By workflow"));
    assert!(md.contains(
        "[CI](https://github.com/DigitalPebble/aspen/blob/HEAD/.github/workflows/ci.yml)"
    ));
}
