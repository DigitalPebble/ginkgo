use ginkgo::estimator::calculate_carbon_impact;
use ginkgo::model::ActionsBill;

fn load_test_bill() -> ActionsBill {
    let json = include_str!("actions_bill.json");
    ActionsBill::from_json(json).unwrap()
}

#[test]
fn sets_energy_and_co2_on_minute_items() {
    let mut bill = load_test_bill();
    calculate_carbon_impact(&mut bill);

    for item in &bill.usage_items {
        if item.unit_type == "Minutes" {
            assert!(
                item.energy_usage_wh.is_some(),
                "energy_usage_wh should be set for {}",
                item.sku
            );
            assert!(
                item.co2eq_g.is_some(),
                "co2eq_g should be set for {}",
                item.sku
            );
            assert!(item.energy_usage_wh.unwrap() > 0.0);
            assert!(item.co2eq_g.unwrap() > 0.0);
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
                "storage items should not have energy_usage_wh"
            );
            assert!(
                item.co2eq_g.is_none(),
                "storage items should not have co2eq_g"
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
    assert!((item.energy_usage_wh.unwrap() - 25.230).abs() < 1e-3);
    assert!((item.co2eq_g.unwrap() - 8.881).abs() < 1e-3);
}

#[test]
fn computes_correct_values_for_linux() {
    let mut bill = load_test_bill();
    calculate_carbon_impact(&mut bill);

    let item = &bill.usage_items[2];
    assert_eq!(item.sku, "Actions Linux");
    assert!((item.energy_usage_wh.unwrap() - 84.197).abs() < 1e-3);
    assert!((item.co2eq_g.unwrap() - 29.637).abs() < 1e-3);
}

#[test]
fn empty_bill_produces_no_errors() {
    let mut bill = ActionsBill::from_json("{\"usageItems\":[]}").unwrap();
    calculate_carbon_impact(&mut bill);
    assert!(bill.usage_items.is_empty());
}
