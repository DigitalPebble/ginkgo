use ginkgo::estimator::calculate_carbon_impact;
use ginkgo::model::ActionsBill;

// Constants matching resources/config.json
const PUE: f64 = 1.16;
const GRID_INTENSITY: f64 = 352.0;

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

#[test]
fn skips_storage_items() {
    let mut bill = load_test_bill();
    calculate_carbon_impact(&mut bill);

    for item in &bill.usage_items {
        if item.unit_type == "GigabyteHours" {
            assert!(
                item.energy_usage_wh.is_none(),
                "energy_usage_wh should not be set for {}",
                item.sku
            );
            assert!(
                item.co2eq_g.is_none(),
                "co2eq_g should not be set for {}",
                item.sku
            );
        }
    }
}

#[test]
fn computes_correct_values_for_linux_arm() {
    let mut bill = load_test_bill();
    calculate_carbon_impact(&mut bill);

    let item = &bill.usage_items[1];
    assert_eq!(item.sku, "Actions Linux ARM");

    let (expected_energy, expected_co2) = expected_values(item.quantity, 2.157); // ubuntu-arm

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

    let (expected_energy, expected_co2) = expected_values(item.quantity, 4.315); // ubuntu

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
