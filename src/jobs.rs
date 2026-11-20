// SPDX-License-Identifier: Apache-2.0

//! Builds usage items from the jobs of GitHub Actions workflow runs
//! (`GET /repos/{owner}/{repo}/actions/runs/{id}/jobs`), as an alternative to
//! the organization billing API.

use crate::model::UsageItem;
use chrono::DateTime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct JobsPage {
    #[serde(default)]
    jobs: Vec<Job>,
}

#[derive(Debug, Deserialize)]
struct Job {
    started_at: Option<String>,
    completed_at: Option<String>,
    #[serde(default)]
    labels: Vec<String>,
}

/// Labels of the standard Intel macOS runners (4 vCPUs), from
/// <https://docs.github.com/en/actions/reference/runners/github-hosted-runners>.
const MACOS_INTEL_LABELS: [&str; 2] = ["macos-15-intel", "macos-26-intel"];

/// Labels of the standard Apple silicon (M1) macOS runners (3 vCPUs), from the same page.
const MACOS_ARM_LABELS: [&str; 4] = ["macos-latest", "macos-15", "macos-26", "xcode-27"];

/// Maps the runner labels of a job to the SKU names used by the billing report.
pub fn sku_for_labels(labels: &[String]) -> String {
    let lower: Vec<String> = labels.iter().map(|l| l.to_lowercase()).collect();
    if lower.iter().any(|l| l == "self-hosted") {
        return "Self-hosted".to_string();
    }
    let any = |needle: &str| lower.iter().any(|l| l.contains(needle));
    let arm = lower.iter().any(|l| l.contains("arm"));
    let is_any = |names: &[&str]| lower.iter().any(|l| names.contains(&l.as_str()));
    if any("ubuntu-slim") {
        "Actions Linux Slim".to_string()
    } else if any("ubuntu") || any("linux") {
        if arm {
            "Actions Linux ARM"
        } else {
            "Actions Linux"
        }
        .to_string()
    } else if any("windows") {
        if arm {
            "Actions Windows ARM"
        } else {
            "Actions Windows"
        }
        .to_string()
    } else if is_any(&MACOS_INTEL_LABELS) {
        "Actions macOS 4-core".to_string()
    } else if is_any(&MACOS_ARM_LABELS) {
        "Actions macOS 3-core".to_string()
    } else {
        labels
            .first()
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

/// Converts one page of the jobs API response into usage items. Each job is
/// rounded up to whole minutes, as in the billing report. Jobs that did not
/// run (skipped, cancelled while queued) are ignored.
pub fn items_from_jobs(json: &str, owner: &str, repo: &str) -> anyhow::Result<Vec<UsageItem>> {
    let page: JobsPage = serde_json::from_str(json)?;
    let mut items = Vec::new();
    for job in page.jobs {
        let (Some(start), Some(end)) = (job.started_at, job.completed_at) else {
            continue;
        };
        let seconds = (DateTime::parse_from_rfc3339(&end)? - DateTime::parse_from_rfc3339(&start)?)
            .num_seconds();
        if seconds <= 0 {
            continue;
        }
        items.push(UsageItem {
            date: start,
            product: "actions".to_string(),
            sku: sku_for_labels(&job.labels),
            quantity: ((seconds + 59) / 60) as f64,
            unit_type: "Minutes".to_string(),
            price_per_unit: 0.0,
            gross_amount: 0.0,
            discount_amount: 0.0,
            net_amount: 0.0,
            organization_name: owner.to_string(),
            repository_name: repo.to_string(),
            workflow_name: None,
            workflow_path: None,
            energy_usage_wh: None,
            co2eq_g: None,
            water_cooling_l: None,
            water_electricity_production_l: None,
        });
    }
    Ok(items)
}
