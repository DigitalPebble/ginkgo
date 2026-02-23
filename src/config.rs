// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;

static CONFIG_JSON: &str = include_str!("../resources/config.json");

pub static INSTANCE: Lazy<Config> = Lazy::new(|| Config::load());

pub struct Config {
    pub runner_power_consumption: HashMap<String, f64>,
    pub grid_carbon_intensity: f64,
    pub pue: f64,
}

impl Config {
    fn load() -> Self {
        let root: Value = serde_json::from_str(CONFIG_JSON).expect("Failed to parse config.json");

        let runners = root["power_consumption"]["runners"]
            .as_object()
            .expect("Missing runners in config");

        let mut runner_power_consumption = HashMap::new();
        for (key, val) in runners {
            let watts = val["value"].as_f64().expect("Missing value for runner");
            runner_power_consumption.insert(key.clone(), watts);
        }

        let grid_carbon_intensity = root["grid_carbon_intensity"]["value"]
            .as_f64()
            .expect("Missing grid_carbon_intensity");

        let pue = root["pue"]["value"].as_f64().expect("Missing pue");

        Config {
            runner_power_consumption,
            grid_carbon_intensity,
            pue,
        }
    }

    pub fn instance() -> &'static Config {
        &INSTANCE
    }
}
