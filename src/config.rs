// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;

static CONFIG_JSON: &str = include_str!("../resources/config.json");

pub static INSTANCE: Lazy<Config> = Lazy::new(Config::load);

/// A SKU billed in gigabyte hours, with the assumptions used to estimate its energy.
pub struct StorageSku {
    /// Energy of the medium it is assumed to use, in Wh per terabyte-hour
    pub wh_per_tb_hour: f64,
    /// Number of copies kept of the billed volume
    pub replication: f64,
}

pub struct Config {
    pub runner_power_consumption: HashMap<String, f64>,
    /// Storage SKUs, keyed by lower case SKU name
    pub storage_skus: HashMap<String, StorageSku>,
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

        let media = root["storage_energy"]
            .as_object()
            .expect("Missing storage_energy in config");

        // energy per medium in Wh per terabyte-hour; the section also carries
        // a description of its own, which has no value
        let mut storage_energy = HashMap::new();
        for (key, val) in media {
            if let Some(wh) = val["value"].as_f64() {
                storage_energy.insert(key.clone(), wh);
            }
        }

        let skus = root["storage"]["skus"]
            .as_object()
            .expect("Missing storage.skus in config");

        let mut storage_skus = HashMap::new();
        for (sku, val) in skus {
            let medium = val["medium"]
                .as_str()
                .unwrap_or_else(|| panic!("Missing storage.skus['{sku}'].medium"));
            let wh_per_tb_hour = *storage_energy
                .get(medium)
                .unwrap_or_else(|| panic!("Unknown storage medium '{medium}' for SKU '{sku}'"));
            let replication = val["replication"]
                .as_f64()
                .unwrap_or_else(|| panic!("Missing storage.skus['{sku}'].replication"));
            storage_skus.insert(
                sku.to_lowercase(),
                StorageSku {
                    wh_per_tb_hour,
                    replication,
                },
            );
        }

        let grid_carbon_intensity = root["grid_carbon_intensity"]["value"]
            .as_f64()
            .expect("Missing grid_carbon_intensity");

        let pue = root["pue"]["value"].as_f64().expect("Missing pue");

        Config {
            runner_power_consumption,
            storage_skus,
            grid_carbon_intensity,
            pue,
        }
    }

    pub fn instance() -> &'static Config {
        &INSTANCE
    }
}
