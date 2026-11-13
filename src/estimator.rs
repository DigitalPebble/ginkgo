// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use crate::model::ActionsBill;

const STORAGE_SKU: &str = "actions storage";
const STORAGE_UNIT: &str = "gigabytehours";
const GB_PER_TB: f64 = 1000.0;

pub fn calculate_carbon_impact(bill: &mut ActionsBill) {
    let config = Config::instance();
    let pue = config.pue;
    let grid_intensity = config.grid_carbon_intensity;

    for item in &mut bill.usage_items {
        let unit = item.unit_type.to_lowercase();

        let energy_wh = if unit == "minutes" {
            let power_watts = match get_runner_power(config, &item.sku) {
                Some(w) => w,
                None => continue,
            };
            power_watts * (item.quantity / 60.0) * pue
        } else if item.sku.to_lowercase() == STORAGE_SKU && unit == STORAGE_UNIT {
            config.ssd_wh_per_tb_hour * (item.quantity / GB_PER_TB) * pue
        } else {
            continue;
        };

        let co2eq_g = energy_wh / 1000.0 * grid_intensity;

        item.energy_usage_wh = Some((energy_wh * 1000.0).round() / 1000.0);
        item.co2eq_g = Some((co2eq_g * 1000.0).round() / 1000.0);
    }
}

fn get_runner_power(config: &Config, sku: &str) -> Option<f64> {
    let runner = match sku {
        "Actions Linux ARM" => "ubuntu-arm",
        "Actions Linux" => "ubuntu",
        _ => return None,
    };
    config.runner_power_consumption.get(runner).copied()
}
