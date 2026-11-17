// SPDX-License-Identifier: Apache-2.0

use crate::model::{ActionsBill, UsageItem};
use std::collections::BTreeMap;

const TOP_REPOSITORIES: usize = 10;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Totals {
    pub energy_wh: f64,
    pub co2eq_g: f64,
    pub minutes: f64,
}

impl Totals {
    fn add(&mut self, energy_wh: f64, co2eq_g: f64, minutes: f64) {
        self.energy_wh += energy_wh;
        self.co2eq_g += co2eq_g;
        self.minutes += minutes;
    }
}

/// Totals over the items that have an estimate.
pub fn totals(bill: &ActionsBill) -> Totals {
    let mut totals = Totals::default();
    for item in &bill.usage_items {
        if let (Some(energy), Some(co2)) = (item.energy_usage_wh, item.co2eq_g) {
            totals.add(energy, co2, item.quantity);
        }
    }
    totals
}

/// Key/value lines to append to the file pointed to by `GITHUB_OUTPUT`.
pub fn step_outputs(bill: &ActionsBill) -> String {
    let t = totals(bill);
    format!(
        "total_co2eq_g={:.3}\ntotal_energy_wh={:.3}\n",
        t.co2eq_g, t.energy_wh
    )
}

/// Markdown report, suited to `GITHUB_STEP_SUMMARY`.
pub fn to_markdown(bill: &ActionsBill) -> String {
    let total = totals(bill);

    let mut by_repo: BTreeMap<&str, Totals> = BTreeMap::new();
    let mut by_sku: BTreeMap<&str, Totals> = BTreeMap::new();
    let mut by_workflow: BTreeMap<(String, String, String), Totals> = BTreeMap::new();
    let mut not_estimated: BTreeMap<&str, f64> = BTreeMap::new();

    for item in &bill.usage_items {
        match (item.energy_usage_wh, item.co2eq_g) {
            (Some(energy), Some(co2)) => {
                if let Some(key) = workflow_key(item) {
                    by_workflow
                        .entry(key)
                        .or_default()
                        .add(energy, co2, item.quantity);
                }
                let repo = if item.repository_name.is_empty() {
                    "(none)"
                } else {
                    item.repository_name.as_str()
                };
                by_repo
                    .entry(repo)
                    .or_default()
                    .add(energy, co2, item.quantity);
                by_sku
                    .entry(item.sku.as_str())
                    .or_default()
                    .add(energy, co2, item.quantity);
            }
            _ if item.unit_type.eq_ignore_ascii_case("minutes") => {
                *not_estimated.entry(item.sku.as_str()).or_default() += item.quantity;
            }
            _ => {}
        }
    }

    let mut md = String::new();
    md.push_str("## 🌳 Aspen - GitHub Actions carbon estimate\n\n");

    if by_repo.is_empty() {
        md.push_str("No usage with an available estimate was found.\n\n");
    } else {
        md.push_str("| Minutes | Energy | CO₂eq |\n|---:|---:|---:|\n");
        md.push_str(&format!(
            "| {:.0} | {} | {} |\n\n",
            total.minutes,
            fmt_energy(total.energy_wh),
            fmt_co2(total.co2eq_g)
        ));

        md.push_str(&breakdown_table(
            "Repository",
            &by_repo,
            Some(TOP_REPOSITORIES),
        ));
        if !by_workflow.is_empty() {
            let rows: BTreeMap<String, Totals> = by_workflow
                .iter()
                .map(|((repo, name, path), t)| (workflow_label(repo, name, path), t.clone()))
                .collect();
            md.push_str(&breakdown_table("Workflow", &rows, Some(TOP_REPOSITORIES)));
        }
        md.push_str(&breakdown_table("Runner SKU", &by_sku, None));
    }

    if !not_estimated.is_empty() {
        md.push_str("### Not estimated\n\n");
        md.push_str("No power model is available yet for these runners, so they are excluded from the totals:\n\n");
        md.push_str("| Runner SKU | Minutes |\n|---|---:|\n");
        for (sku, minutes) in &not_estimated {
            md.push_str(&format!("| {} | {:.0} |\n", sku, minutes));
        }
        md.push('\n');
    }

    md.push_str(
        "<sub>Estimates by [Aspen](https://github.com/DigitalPebble/aspen) \
         based on billed minutes. Need help with GreenOps? \
         Get in touch with [DigitalPebble](https://digitalpebble.com/).</sub>\n",
    );
    md
}

/// Workflows are grouped by file: (`owner/repo`, name, path). Items without a
/// workflow path fall back to the name alone.
fn workflow_key(item: &UsageItem) -> Option<(String, String, String)> {
    let name = item.workflow_name.as_deref();
    let path = item.workflow_path.as_deref();
    let repo = if item.organization_name.is_empty() || item.repository_name.is_empty() {
        String::new()
    } else {
        format!("{}/{}", item.organization_name, item.repository_name)
    };
    Some((
        repo,
        name.or(path)?.to_string(),
        path.unwrap_or("").to_string(),
    ))
}

/// Workflow name linked to its file in the repository, when the path is known.
fn workflow_label(repo: &str, name: &str, path: &str) -> String {
    let name = name.replace('|', "\\|");
    if path.is_empty() || repo.is_empty() {
        return name;
    }
    format!("[{}]({})", name, workflow_url(repo, path))
}

fn workflow_url(repo: &str, path: &str) -> String {
    format!("https://github.com/{}/blob/HEAD/{}", repo, path)
}

fn breakdown_table<K: AsRef<str> + Ord>(
    label: &str,
    rows: &BTreeMap<K, Totals>,
    limit: Option<usize>,
) -> String {
    let mut sorted: Vec<(&K, &Totals)> = rows.iter().collect();
    sorted.sort_by(|a, b| {
        b.1.co2eq_g
            .partial_cmp(&a.1.co2eq_g)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(b.0))
    });

    let mut out = match limit {
        Some(n) if sorted.len() > n => format!("### Top {} by {}\n\n", n, label.to_lowercase()),
        _ => format!("### By {}\n\n", label.to_lowercase()),
    };
    out.push_str(&format!(
        "| {} | Minutes | Energy | CO₂eq |\n|---|---:|---:|---:|\n",
        label
    ));

    let shown = limit.unwrap_or(sorted.len());
    for (name, t) in sorted.iter().take(shown) {
        out.push_str(&format!(
            "| {} | {:.0} | {} | {} |\n",
            name.as_ref(),
            t.minutes,
            fmt_energy(t.energy_wh),
            fmt_co2(t.co2eq_g)
        ));
    }
    if sorted.len() > shown {
        let rest = sorted.len() - shown;
        out.push_str(&format!("| _{} more…_ | | | |\n", rest));
    }
    out.push('\n');
    out
}

/// Colour of the badge when the data comes from the jobs API (no key needed): less accurate.
pub const BADGE_COLOR_JOBS: &str = "c0c0c0"; // silver
/// Colour of the badge when the data comes from the billing API (key needed): more accurate.
pub const BADGE_COLOR_BILLING: &str = "ffd700"; // gold

/// Badge in the [shields.io endpoint](https://shields.io/badges/endpoint-badge) format.
/// `period` (e.g. "30d") is added to the label when not empty.
pub fn badge_json(bill: &ActionsBill, period: &str, color: &str) -> String {
    let grams = totals(bill).co2eq_g;
    let message = if grams >= 1000.0 {
        format!("{:.2} kgCO2e", grams / 1000.0)
    } else {
        format!("{:.1} gCO2e", grams)
    };
    let label = if period.is_empty() {
        "CI CO2eq".to_string()
    } else {
        format!("CI CO2eq ({})", period)
    };
    serde_json::json!({
        "schemaVersion": 1,
        "label": label,
        "message": message,
        "color": color,
    })
    .to_string()
}

pub fn fmt_co2(grams: f64) -> String {
    if grams >= 1000.0 {
        format!("{:.2} kg", grams / 1000.0)
    } else {
        format!("{:.1} g", grams)
    }
}

pub fn fmt_energy(wh: f64) -> String {
    if wh >= 1000.0 {
        format!("{:.2} kWh", wh / 1000.0)
    } else {
        format!("{:.1} Wh", wh)
    }
}
