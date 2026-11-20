// SPDX-License-Identifier: Apache-2.0

use aspen::estimator::calculate_carbon_impact;
use aspen::model::ActionsBill;

// Constants matching resources/config.json
const PUE: f64 = 1.16;
const GRID_INTENSITY: f64 = 383.0;

fn load_test_bill() -> ActionsBill {
    let json = include_str!("actions_bill.json");
    ActionsBill::from_json(json).unwrap()
}

fn assert_close(label: &str, actual: f64, expected: f64) {
    let diff = (actual - expected).abs();
    assert!(
        diff < 1e-3,
        "{label}: expected {expected:.6}, got {actual:.6} (diff: {diff:.6})"
    );
}

/// Computes the expected (energy_wh, co2_g) for a given quantity and runner power,
/// applying the same rounding the estimator uses.
fn expected_values(minutes: f64, power_w: f64) -> (f64, f64) {
    let energy_wh = power_w * (minutes / 60.0) * PUE;
    let co2_g = (energy_wh / 1000.0) * GRID_INTENSITY;
    (
        (energy_wh * 1000.0).round() / 1000.0,
        (co2_g * 1000.0).round() / 1000.0,
    )
}

#[test]
fn sets_energy_and_co2_on_minute_items() {
    let mut bill = load_test_bill();
    calculate_carbon_impact(&mut bill);

    for item in &bill.usage_items {
        if item.unit_type == "Minutes" {
            let energy = item
                .energy_usage_wh
                .unwrap_or_else(|| panic!("energy_usage_wh not set for {}", item.sku));
            let co2 = item
                .co2eq_g
                .unwrap_or_else(|| panic!("co2eq_g not set for {}", item.sku));
            assert!(
                energy > 0.0,
                "energy_usage_wh should be positive for {}",
                item.sku
            );
            assert!(co2 > 0.0, "co2eq_g should be positive for {}", item.sku);
        }
    }
}

const SSD_WH_PER_TB_HOUR: f64 = 1.2;
const HDD_WH_PER_TB_HOUR: f64 = 0.65;
const ACTIONS_STORAGE_REPLICATION: f64 = 1.0;
const PACKAGES_STORAGE_REPLICATION: f64 = 3.0;

/// Builds a single usage item of `sku` billed in gigabyte hours.
fn storage_bill(product: &str, sku: &str, gigabyte_hours: f64) -> ActionsBill {
    let json = format!(
        r#"{{"usageItems":[{{"date":"2026-01-01T00:00:00Z","product":"{product}",
        "sku":"{sku}","quantity":{gigabyte_hours},"unitType":"GigabyteHours","pricePerUnit":0.0,
        "grossAmount":0.0,"discountAmount":0.0,"netAmount":0.0,
        "organizationName":"o","repositoryName":"r"}}]}}"#
    );
    ActionsBill::from_json(&json).unwrap()
}

fn assert_storage_values(sku: &str, wh_per_tb_hour: f64, replication: f64) {
    let gigabyte_hours = 2.533244733;
    let mut bill = storage_bill("packages", sku, gigabyte_hours);
    calculate_carbon_impact(&mut bill);

    let item = &bill.usage_items[0];
    let energy_wh = wh_per_tb_hour * (gigabyte_hours / 1000.0) * replication * PUE;
    let co2_g = energy_wh / 1000.0 * GRID_INTENSITY;

    assert_close(
        &format!("{sku} energy_usage_wh"),
        item.energy_usage_wh.expect("energy_usage_wh not set"),
        (energy_wh * 1000.0).round() / 1000.0,
    );
    assert_close(
        &format!("{sku} co2eq_g"),
        item.co2eq_g.expect("co2eq_g not set"),
        (co2_g * 1000.0).round() / 1000.0,
    );
}

#[test]
fn computes_correct_values_for_storage() {
    let mut bill = load_test_bill();
    calculate_carbon_impact(&mut bill);

    let item = &bill.usage_items[0];
    assert_eq!(item.sku, "Actions storage");
    assert_eq!(item.unit_type, "GigabyteHours");

    let energy_wh =
        SSD_WH_PER_TB_HOUR * (item.quantity / 1000.0) * ACTIONS_STORAGE_REPLICATION * PUE;
    let co2_g = energy_wh / 1000.0 * GRID_INTENSITY;

    assert_close(
        "energy_usage_wh",
        item.energy_usage_wh.expect("energy_usage_wh not set"),
        (energy_wh * 1000.0).round() / 1000.0,
    );
    assert_close(
        "co2eq_g",
        item.co2eq_g.expect("co2eq_g not set"),
        (co2_g * 1000.0).round() / 1000.0,
    );
}

#[test]
fn computes_correct_values_for_packages_storage() {
    assert_storage_values(
        "Packages storage",
        HDD_WH_PER_TB_HOUR,
        PACKAGES_STORAGE_REPLICATION,
    );
}

#[test]
fn matches_storage_sku_regardless_of_case() {
    assert_storage_values(
        "PACKAGES STORAGE",
        HDD_WH_PER_TB_HOUR,
        PACKAGES_STORAGE_REPLICATION,
    );
}

#[test]
fn skips_storage_sku_missing_from_config() {
    let mut bill = storage_bill("copilot", "Copilot storage", 1000.0);
    calculate_carbon_impact(&mut bill);

    let item = &bill.usage_items[0];
    assert!(item.energy_usage_wh.is_none());
    assert!(item.co2eq_g.is_none());
}

#[test]
fn skips_storage_with_unexpected_unit() {
    let json = r#"{"usageItems":[{"date":"2026-01-01T00:00:00Z","product":"actions",
        "sku":"Actions storage","quantity":1000.0,"unitType":"Minutes","pricePerUnit":0.0,
        "grossAmount":0.0,"discountAmount":0.0,"netAmount":0.0,
        "organizationName":"o","repositoryName":"r"},
        {"date":"2026-01-01T00:00:00Z","product":"actions",
        "sku":"Actions storage","quantity":1000.0,"unitType":"Bytes","pricePerUnit":0.0,
        "grossAmount":0.0,"discountAmount":0.0,"netAmount":0.0,
        "organizationName":"o","repositoryName":"r"}]}"#;
    let mut bill = ActionsBill::from_json(json).unwrap();
    calculate_carbon_impact(&mut bill);
    for item in &bill.usage_items {
        assert!(item.energy_usage_wh.is_none());
        assert!(item.co2eq_g.is_none());
    }
}

#[test]
fn computes_correct_values_for_linux_arm() {
    let mut bill = load_test_bill();
    calculate_carbon_impact(&mut bill);

    let item = &bill.usage_items[1];
    assert_eq!(item.sku, "Actions Linux ARM");

    let (expected_energy, expected_co2) = expected_values(item.quantity, 6.66); // ubuntu-arm

    assert_close(
        "energy_usage_wh",
        item.energy_usage_wh.expect("energy_usage_wh not set"),
        expected_energy,
    );
    assert_close(
        "co2eq_g",
        item.co2eq_g.expect("co2eq_g not set"),
        expected_co2,
    );
}

#[test]
fn computes_correct_values_for_linux() {
    let mut bill = load_test_bill();
    calculate_carbon_impact(&mut bill);

    let item = &bill.usage_items[2];
    assert_eq!(item.sku, "Actions Linux");

    let (expected_energy, expected_co2) = expected_values(item.quantity, 11.52); // ubuntu

    assert_close(
        "energy_usage_wh",
        item.energy_usage_wh.expect("energy_usage_wh not set"),
        expected_energy,
    );
    assert_close(
        "co2eq_g",
        item.co2eq_g.expect("co2eq_g not set"),
        expected_co2,
    );
}

#[test]
fn empty_bill_produces_no_errors() {
    let mut bill = ActionsBill::from_json("{\"usageItems\":[]}").unwrap();
    calculate_carbon_impact(&mut bill);
    assert!(bill.usage_items.is_empty());
}

#[test]
fn splits_water_between_cooling_and_electricity() {
    let mut bill = load_test_bill();
    calculate_carbon_impact(&mut bill);

    let item = bill
        .usage_items
        .iter()
        .find(|i| i.sku == "Actions Linux")
        .expect("no Actions Linux item");
    let energy_kwh = item.energy_usage_wh.expect("energy not set") / 1000.0;

    let cooling = item.water_cooling_l.expect("water_cooling_l not set");
    let electricity = item
        .water_electricity_production_l
        .expect("water_electricity_production_l not set");

    assert!((cooling - energy_kwh * 0.34).abs() < 1e-4);
    assert!((electricity - energy_kwh * 3.5781).abs() < 1e-4);
}

#[test]
fn no_water_without_energy_estimate() {
    let mut bill = load_test_bill();
    calculate_carbon_impact(&mut bill);
    for item in bill
        .usage_items
        .iter()
        .filter(|i| i.energy_usage_wh.is_none())
    {
        assert!(item.water_cooling_l.is_none());
        assert!(item.water_electricity_production_l.is_none());
    }
}
