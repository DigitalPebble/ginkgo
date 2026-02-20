use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageItem {
    pub date: String,
    pub product: String,
    pub sku: String,
    pub quantity: f64,
    pub unit_type: String,
    pub price_per_unit: f64,
    pub gross_amount: f64,
    pub discount_amount: f64,
    pub net_amount: f64,
    pub organization_name: String,
    pub repository_name: String,

    #[serde(rename = "energy_usage_wh", skip_serializing_if = "Option::is_none")]
    pub energy_usage_wh: Option<f64>,

    #[serde(rename = "co2eq_g", skip_serializing_if = "Option::is_none")]
    pub co2eq_g: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActionsBill {
    #[serde(default)]
    pub usage_items: Vec<UsageItem>,
}

impl ActionsBill {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let json = fs::read_to_string(path)?;
        Self::from_json(&json)
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

}
