// SPDX-License-Identifier: Apache-2.0

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

/// Splits an `owner/name` repository reference.
pub fn parse_repository(repo: &str) -> anyhow::Result<(String, String)> {
    match repo.split_once('/') {
        Some((o, n)) if !o.is_empty() && !n.is_empty() && !n.contains('/') => {
            Ok((o.to_string(), n.to_string()))
        }
        _ => anyhow::bail!("Repository must be in the form owner/name, got '{}'", repo),
    }
}

impl ActionsBill {
    /// Keeps only the usage items of the given repository (case-insensitive).
    pub fn retain_repository(&mut self, owner: &str, name: &str) {
        self.usage_items.retain(|i| {
            i.repository_name.eq_ignore_ascii_case(name)
                && i.organization_name.eq_ignore_ascii_case(owner)
        });
    }

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

    #[allow(dead_code)]
    pub fn to_file(&self, path: &Path) -> anyhow::Result<()> {
        let json = self.to_json()?;
        fs::write(path, json)?;
        Ok(())
    }
}
