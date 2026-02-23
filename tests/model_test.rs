// SPDX-License-Identifier: Apache-2.0

use ginkgo::model::ActionsBill;

fn load_test_json() -> &'static str {
    include_str!("actions_bill.json")
}

#[test]
fn deserialise_from_json() {
    let bill = ActionsBill::from_json(load_test_json()).unwrap();

    assert_eq!(bill.usage_items.len(), 6);

    let first = &bill.usage_items[0];
    assert_eq!(first.date, "2026-01-01T00:00:00Z");
    assert_eq!(first.product, "actions");
    assert_eq!(first.sku, "Actions storage");
    assert!((first.quantity - 1.978155783).abs() < 1e-9);
    assert_eq!(first.unit_type, "GigabyteHours");
    assert!((first.price_per_unit - 0.00033602).abs() < 1e-9);
    assert!((first.gross_amount - 0.000664386).abs() < 1e-9);
    assert!((first.discount_amount - 0.000664386).abs() < 1e-9);
    assert!((first.net_amount - 0.0).abs() < 1e-9);
    assert_eq!(first.organization_name, "DigitalPebble");
    assert_eq!(first.repository_name, "spruce");
}

#[test]
fn serialise_to_json() {
    let bill = ActionsBill::from_json(load_test_json()).unwrap();
    let serialised = bill.to_json().unwrap();
    let round_tripped = ActionsBill::from_json(&serialised).unwrap();

    assert_eq!(bill.usage_items.len(), round_tripped.usage_items.len());

    for (original, restored) in bill
        .usage_items
        .iter()
        .zip(round_tripped.usage_items.iter())
    {
        assert_eq!(original.date, restored.date);
        assert_eq!(original.sku, restored.sku);
        assert!((original.quantity - restored.quantity).abs() < 1e-9);
        assert_eq!(original.unit_type, restored.unit_type);
        assert_eq!(original.repository_name, restored.repository_name);
    }
}

#[test]
fn read_and_write_file() {
    let bill = ActionsBill::from_json(load_test_json()).unwrap();

    let dir = std::env::temp_dir().join("ginkgo_test");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("output.json");

    bill.to_file(&file).unwrap();
    assert!(file.exists());

    let from_file = ActionsBill::from_file(&file).unwrap();
    assert_eq!(bill.usage_items.len(), from_file.usage_items.len());
    assert_eq!(bill.usage_items[0].sku, from_file.usage_items[0].sku);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn empty_usage_items() {
    let bill = ActionsBill::from_json("{\"usageItems\":[]}").unwrap();
    assert!(bill.usage_items.is_empty());
}

#[test]
fn missing_usage_items_defaults_to_empty() {
    let bill = ActionsBill::from_json("{}").unwrap();
    assert!(bill.usage_items.is_empty());
}
