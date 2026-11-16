// SPDX-License-Identifier: Apache-2.0

mod config;
mod estimator;
mod jobs;
mod model;
mod summary;

use anyhow::{bail, Context, Result};
use clap::Parser;
use model::ActionsBill;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Clone, Copy, clap::ValueEnum)]
enum Source {
    /// Organization billing API (needs organization Administration read)
    Billing,
    /// Jobs of the repository's workflow runs (needs only Actions read)
    Jobs,
}

/// Aspen - GitHub Actions Carbon Estimator
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to a local billing JSON file (instead of fetching from GitHub API)
    #[arg(short, long)]
    file: Option<String>,

    /// GitHub token with organization Administration (read) permission
    #[arg(short, long, env = "INPUT_GITHUB_TOKEN")]
    token: Option<String>,

    /// GitHub organization name
    #[arg(short, long, env = "INPUT_ORGANIZATION")]
    organization: Option<String>,

    /// Restrict the report to a single repository ("owner/name"); the organization is derived from it
    #[arg(short, long, env = "INPUT_REPOSITORY")]
    repository: Option<String>,

    /// Where to get the usage data from: the organization billing API or the repository's workflow jobs
    #[arg(long, env = "INPUT_SOURCE", value_enum, default_value = "billing")]
    source: Source,

    /// Number of days of workflow runs to analyse (jobs source only)
    #[arg(long, env = "INPUT_DAYS", default_value_t = 30)]
    days: u32,

    /// Year of the billing usage report (billing source only; defaults to the current period)
    #[arg(long, env = "INPUT_YEAR", value_parser = clap::value_parser!(u32).range(2000..=9999))]
    year: Option<u32>,

    /// Month (1-12) of the billing usage report (billing source only)
    #[arg(long, env = "INPUT_MONTH", value_parser = clap::value_parser!(u32).range(1..=12))]
    month: Option<u32>,

    /// Day (1-31) of the billing usage report (billing source only)
    #[arg(long, env = "INPUT_DAY", value_parser = clap::value_parser!(u32).range(1..=31))]
    day: Option<u32>,

    /// Path where the carbon estimate report will be saved
    #[arg(
        short = 'O',
        long,
        env = "INPUT_OUTPUT_PATH",
        default_value = "./carbon-estimate.json"
    )]
    output: String,

    /// Path where a shields.io endpoint badge (JSON) will be written
    #[arg(long, env = "INPUT_BADGE_PATH")]
    badge: Option<String>,

    /// File to append a markdown summary to (totals, per repository and per SKU)
    #[arg(short, long, env = "GITHUB_STEP_SUMMARY")]
    summary: Option<String>,

    /// File to append step outputs to (set automatically by GitHub Actions)
    #[arg(long, env = "GITHUB_OUTPUT", hide = true)]
    github_output: Option<String>,
}

fn append_to_file(path: &str, content: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open {}", path))?;
    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write to {}", path))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let filter_mode = cli.file.is_none()
        && cli.token.is_none()
        && cli.organization.is_none()
        && cli.repository.is_none();

    let repo = cli
        .repository
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(model::parse_repository)
        .transpose()?;

    // The badge is gold when the data is billing data (from the billing API, which
    // needs a key with organization access, or from a file) and silver for the
    // jobs source, which is less accurate.
    let mut badge_color = summary::BADGE_COLOR_JOBS;
    let mut badge_period = String::new();

    let bill = if filter_mode {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .context("Failed to read from stdin")?;
        ActionsBill::from_json(&input)?
    } else if let Some(input_path) = &cli.file {
        println!("Loading billing data from file: {}", input_path);
        let bill = ActionsBill::from_file(Path::new(input_path))?;
        println!("Loaded {} usage items", bill.usage_items.len());
        badge_color = summary::BADGE_COLOR_BILLING;
        bill
    } else if matches!(cli.source, Source::Jobs) {
        if cli.year.is_some() || cli.month.is_some() || cli.day.is_some() {
            bail!("--year, --month and --day are only supported with --source billing; use --days with --source jobs");
        }
        let token = cli
            .token
            .as_deref()
            .filter(|s| !s.is_empty())
            .context("GitHub token is required (set --token or INPUT_GITHUB_TOKEN)")?;
        let (owner, name) = repo
            .as_ref()
            .context("--repository is required with --source jobs")?;
        println!("Fetching workflow jobs for {}/{}", owner, name);
        let bill = fetch_jobs_usage(token, owner, name, cli.days)?;
        badge_period = format!("{}d", cli.days);
        println!("Loaded {} usage items", bill.usage_items.len());
        bill
    } else {
        let token = cli.token.as_deref().filter(|s| !s.is_empty()).context(
            "GitHub token is required when not using --file (set --token or INPUT_GITHUB_TOKEN)",
        )?;
        let organization = repo
            .as_ref()
            .map(|(o, _)| o.as_str())
            .or(cli.organization.as_deref().filter(|s| !s.is_empty()))
            .context("Organization is required when not using --file (set --organization or INPUT_ORGANIZATION)")?;
        println!("Fetching billing data for organization: {}", organization);
        let bill = fetch_billing_usage(token, organization, cli.year, cli.month, cli.day)?;
        badge_color = summary::BADGE_COLOR_BILLING;
        badge_period = match (cli.year, cli.month, cli.day) {
            (Some(y), Some(m), Some(d)) => format!("{y}-{m:02}-{d:02}"),
            (Some(y), Some(m), None) => format!("{y}-{m:02}"),
            (Some(y), None, None) => y.to_string(),
            _ => String::new(),
        };
        println!("Loaded {} usage items", bill.usage_items.len());
        bill
    };

    let mut bill = bill;
    if let Some((owner, name)) = &repo {
        bill.retain_repository(owner, name);
        println!(
            "Kept {} usage items for {}/{}",
            bill.usage_items.len(),
            owner,
            name
        );
    }
    if !filter_mode {
        println!("Estimating carbon footprint...");
    }
    estimator::calculate_carbon_impact(&mut bill);

    let json = bill.to_json()?;

    if let Some(path) = cli.summary.as_deref().filter(|s| !s.is_empty()) {
        append_to_file(path, &summary::to_markdown(&bill))?;
    }
    if let Some(path) = cli.github_output.as_deref().filter(|s| !s.is_empty()) {
        append_to_file(path, &summary::step_outputs(&bill))?;
    }

    if filter_mode {
        println!("{}", json);
    } else {
        if let Some(parent) = Path::new(&cli.output).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&cli.output, &json)?;
        println!("Carbon estimate saved to: {}", cli.output);
        if let Some(path) = cli.badge.as_deref().filter(|s| !s.is_empty()) {
            if let Some(parent) = Path::new(path).parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, summary::badge_json(&bill, &badge_period, badge_color))?;
            println!("Badge saved to: {}", path);
        }
    }

    Ok(())
}

fn fetch_billing_usage(
    token: &str,
    organization: &str,
    year: Option<u32>,
    month: Option<u32>,
    day: Option<u32>,
) -> Result<ActionsBill> {
    let mut url = format!(
        "https://api.github.com/organizations/{}/settings/billing/usage",
        organization
    );
    let params: Vec<String> = [("year", year), ("month", month), ("day", day)]
        .iter()
        .filter_map(|(k, v)| v.map(|v| format!("{}={}", k, v)))
        .collect();
    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }

    let client = reqwest::blocking::Client::new();
    let request = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", format!("Aspen/{}", organization))
        .build()
        .context("Failed to build request")?;

    let headers = request.headers().clone();
    let response = client
        .execute(request)
        .context("Failed to send request to GitHub API")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        let curl = headers
            .iter()
            .fold(format!("curl -s \"{}\"", url), |acc, (k, v)| {
                format!("{} -H \"{}: {}\"", acc, k, v.to_str().unwrap_or("?"))
            });
        bail!(
            "Failed to fetch Actions billing: {} - {}\n{}",
            status.as_u16(),
            body,
            curl
        );
    }

    let body = response.text()?;
    ActionsBill::from_json(&body)
}

const MAX_RUN_PAGES: u32 = 50;

fn get_json(client: &reqwest::blocking::Client, token: &str, url: &str) -> Result<String> {
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "Aspen")
        .send()
        .with_context(|| format!("Failed to send request to {}", url))?;
    let status = response.status();
    let body = response.text()?;
    if !status.is_success() {
        bail!(
            "GitHub API request {} failed: {} - {}",
            url,
            status.as_u16(),
            body
        );
    }
    Ok(body)
}

fn fetch_jobs_usage(token: &str, owner: &str, repo: &str, days: u32) -> Result<ActionsBill> {
    #[derive(serde::Deserialize)]
    struct Runs {
        #[serde(default)]
        workflow_runs: Vec<Run>,
    }
    #[derive(serde::Deserialize)]
    struct Run {
        id: u64,
    }

    let client = reqwest::blocking::Client::new();
    let since = (chrono::Utc::now() - chrono::Duration::days(days as i64)).format("%Y-%m-%d");
    let base = format!("https://api.github.com/repos/{}/{}/actions", owner, repo);
    let mut bill = ActionsBill::default();

    for page in 1..=MAX_RUN_PAGES {
        let url = format!(
            "{}/runs?status=completed&created=>={}&per_page=100&page={}",
            base, since, page
        );
        let runs: Runs = serde_json::from_str(&get_json(&client, token, &url)?)?;
        let count = runs.workflow_runs.len();
        for run in runs.workflow_runs {
            for jobs_page in 1.. {
                let url = format!(
                    "{}/runs/{}/jobs?filter=all&per_page=100&page={}",
                    base, run.id, jobs_page
                );
                let body = get_json(&client, token, &url)?;
                let items = jobs::items_from_jobs(&body, owner, repo)?;
                let total: usize = serde_json::from_str::<serde_json::Value>(&body)?["jobs"]
                    .as_array()
                    .map_or(0, |a| a.len());
                bill.usage_items.extend(items);
                if total < 100 {
                    break;
                }
            }
        }
        if count < 100 {
            return Ok(bill);
        }
    }
    eprintln!(
        "Warning: stopped after {} pages of workflow runs; reduce --days for complete data",
        MAX_RUN_PAGES
    );
    Ok(bill)
}
