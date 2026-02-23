// SPDX-License-Identifier: Apache-2.0

mod config;
mod estimator;
mod model;

use anyhow::{bail, Context, Result};
use clap::Parser;
use model::ActionsBill;
use std::fs;
use std::path::Path;

/// Ginkgo - GitHub Actions Carbon Estimator
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to a local billing JSON file (instead of fetching from GitHub API)
    #[arg(short, long)]
    file: Option<String>,

    /// GitHub token with billing:read permission
    #[arg(short, long, env = "INPUT_GITHUB_TOKEN")]
    token: Option<String>,

    /// GitHub organization name
    #[arg(short, long, env = "INPUT_ORGANIZATION")]
    organization: Option<String>,

    /// Path where the carbon estimate report will be saved
    #[arg(
        short = 'O',
        long,
        env = "INPUT_OUTPUT_PATH",
        default_value = "./carbon-estimate.json"
    )]
    output: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let bill = if let Some(input_path) = &cli.file {
        println!("Loading billing data from file: {}", input_path);
        let bill = ActionsBill::from_file(Path::new(input_path))?;
        println!("Loaded {} usage items", bill.usage_items.len());
        bill
    } else {
        let token = cli.token.as_deref().filter(|s| !s.is_empty()).context(
            "GitHub token is required when not using --file (set --token or INPUT_GITHUB_TOKEN)",
        )?;
        let organization = cli
            .organization
            .as_deref()
            .filter(|s| !s.is_empty())
            .context("Organization is required when not using --file (set --organization or INPUT_ORGANIZATION)")?;
        println!("Fetching billing data for organization: {}", organization);
        let bill = fetch_billing_usage(token, organization)?;
        println!("Loaded {} usage items", bill.usage_items.len());
        bill
    };

    let mut bill = bill;
    println!("Estimating carbon footprint...");
    estimator::calculate_carbon_impact(&mut bill);

    if let Some(parent) = Path::new(&cli.output).parent() {
        fs::create_dir_all(parent)?;
    }
    let json = bill.to_json()?;
    fs::write(&cli.output, json)?;

    println!("Carbon estimate saved to: {}", cli.output);
    Ok(())
}

fn fetch_billing_usage(token: &str, organization: &str) -> Result<ActionsBill> {
    let url = format!(
        "https://api.github.com/organizations/{}/settings/billing/usage",
        organization
    );

    let client = reqwest::blocking::Client::new();
    let request = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", format!("Ginkgo/{}", organization))
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
