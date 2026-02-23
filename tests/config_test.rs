use ginkgo::config::Config;

#[test]
fn loads_successfully() {
    let config = Config::instance();
    assert!(config.grid_carbon_intensity > 0.0);
}

#[test]
fn grid_carbon_intensity() {
    assert!((Config::instance().grid_carbon_intensity - 352.0).abs() < 1e-9);
}

#[test]
fn pue() {
    assert!((Config::instance().pue - 1.16).abs() < 1e-9);
}

#[test]
fn runner_power_consumption_contains_all_runners() {
    let runners = &Config::instance().runner_power_consumption;
    assert_eq!(runners.len(), 7);
    assert!((runners["ubuntu"] - 4.315).abs() < 1e-9);
    assert!((runners["ubuntu-slim"] - 1.078).abs() < 1e-9);
    assert!((runners["ubuntu-arm"] - 2.157).abs() < 1e-9);
    assert!((runners["windows"] - 4.746).abs() < 1e-9);
    assert!((runners["windows-arm"] - 2.372).abs() < 1e-9);
    assert!((runners["macos"] + 1.0).abs() < 1e-9);
    assert!((runners["macos-13"] + 1.0).abs() < 1e-9);
}
