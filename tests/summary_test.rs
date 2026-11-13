// SPDX-License-Identifier: Apache-2.0

use ginkgo::estimator::calculate_carbon_impact;
use ginkgo::model::ActionsBill;
use ginkgo::summary::{fmt_co2, fmt_energy, step_outputs, to_markdown, totals};

fn estimated_bill() -> ActionsBill {
    let mut bill = ActionsBill::from_json(include_str!("actions_bill.json")).unwrap();
    calculate_carbon_impact(&mut bill);
    bill
}

#[test]
fn totals_sum_estimated_items_only() {
    let bill = estimated_bill();
    let t = totals(&bill);

    let expected_co2: f64 = bill.usage_items.iter().filter_map(|i| i.co2eq_g).sum();
    let expected_energy: f64 = bill
        .usage_items
        .iter()
        .filter_map(|i| i.energy_usage_wh)
        .sum();

    assert!(t.co2eq_g > 0.0);
    assert!((t.co2eq_g - expected_co2).abs() < 1e-9);
    assert!((t.energy_wh - expected_energy).abs() < 1e-9);
}

#[test]
fn step_outputs_contains_totals() {
    let out = step_outputs(&estimated_bill());
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("total_co2eq_g="));
    assert!(lines[1].starts_with("total_energy_wh="));
}

#[test]
fn markdown_has_sections() {
    let md = to_markdown(&estimated_bill());
    assert!(md.contains("Ginkgo"));
    assert!(md.contains("By repository"));
    assert!(md.contains("By runner sku"));
    assert!(md.contains("spruce"));
}

#[test]
fn markdown_for_empty_bill() {
    let bill = ActionsBill::from_json("{\"usageItems\":[]}").unwrap();
    let md = to_markdown(&bill);
    assert!(md.contains("No usage with an available estimate"));
    assert_eq!(step_outputs(&bill), "total_co2eq_g=0.000\ntotal_energy_wh=0.000\n");
}

#[test]
fn lists_runners_without_estimate() {
    let json = r#"{"usageItems":[{"date":"2026-01-01T00:00:00Z","product":"actions",
        "sku":"Actions macOS 3-core","quantity":10.0,"unitType":"Minutes","pricePerUnit":0.0,
        "grossAmount":0.0,"discountAmount":0.0,"netAmount":0.0,
        "organizationName":"o","repositoryName":"r"}]}"#;
    let mut bill = ActionsBill::from_json(json).unwrap();
    calculate_carbon_impact(&mut bill);
    let md = to_markdown(&bill);
    assert!(md.contains("Not estimated"));
    assert!(md.contains("Actions macOS 3-core"));
}

#[test]
fn formats_units() {
    assert_eq!(fmt_co2(12.34), "12.3 g");
    assert_eq!(fmt_co2(2500.0), "2.50 kg");
    assert_eq!(fmt_energy(5.0), "5.0 Wh");
    assert_eq!(fmt_energy(1500.0), "1.50 kWh");
}
