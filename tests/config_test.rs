// SPDX-License-Identifier: Apache-2.0

use aspen::config::Config;

fn assert_close(label: &str, actual: f64, expected: f64) {
    let diff = (actual - expected).abs();
    assert!(
        diff < 1e-9,
        "{label}: expected {expected:.9}, got {actual:.9} (diff: {diff:.9})"
    );
}

#[test]
fn loads_successfully() {
    let config = Config::instance();
    assert!(config.grid_carbon_intensity > 0.0);
}

#[test]
fn grid_carbon_intensity() {
    assert_close(
        "grid_carbon_intensity",
        Config::instance().grid_carbon_intensity,
        383.0,
    );
}

#[test]
fn pue() {
    assert_close("pue", Config::instance().pue, 1.16);
}

#[test]
fn runner_power_consumption_contains_all_runners() {
    let runners = &Config::instance().runner_power_consumption;
    assert_eq!(
        runners.len(),
        7,
        "expected 7 runner types, got {}",
        runners.len()
    );

    let cases = [
        ("ubuntu", 11.52),
        ("ubuntu-slim", 2.88),
        ("ubuntu-arm", 6.66),
        ("windows", 12.67),
        ("windows-arm", 7.33),
        ("macos", -1.0),
        ("macos-13", -1.0),
    ];

    for (name, expected) in cases {
        let actual = *runners
            .get(name)
            .unwrap_or_else(|| panic!("runner '{name}' missing from config"));
        assert_close(&format!("runners[{name}]"), actual, expected);
    }
}

#[test]
fn storage_skus_resolve_their_medium_and_replication() {
    let skus = &Config::instance().storage_skus;
    assert_eq!(skus.len(), 2, "expected 2 storage SKUs, got {}", skus.len());

    // SKUs are keyed in lower case, and carry the energy of their medium
    let cases = [
        ("actions storage", 1.2, 1.0),
        ("packages storage", 0.65, 3.0),
    ];

    for (sku, wh_per_tb_hour, replication) in cases {
        let storage = skus
            .get(sku)
            .unwrap_or_else(|| panic!("storage SKU '{sku}' missing from config"));
        assert_close(
            &format!("{sku} Wh per terabyte-hour"),
            storage.wh_per_tb_hour,
            wh_per_tb_hour,
        );
        assert_close(
            &format!("{sku} replication"),
            storage.replication,
            replication,
        );
    }
}
