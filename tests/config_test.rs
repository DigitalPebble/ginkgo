use ginkgo::config::Config;

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
        352.0,
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
        ("ubuntu", 4.315),
        ("ubuntu-slim", 1.078),
        ("ubuntu-arm", 2.157),
        ("windows", 4.746),
        ("windows-arm", 2.372),
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
