mod config;
mod estimator;
mod model;

use anyhow::{bail, Context, Result};
use model::ActionsBill;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let bill = if args.len() == 2 {
        let input_path = &args[1];
        println!("Loading billing data from file: {}", input_path);
        let bill = ActionsBill::from_file(Path::new(input_path))?;
        println!("Loaded {} usage items", bill.usage_items.len());
        bill
    } else {
        let token = get_env_required("INPUT_GITHUB_TOKEN")?;
        let organization = get_env_required("INPUT_ORGANIZATION")?;
        println!("Fetching billing data for organization: {}", organization);
        let bill = fetch_billing_usage(&token, &organization)?;
        println!("Loaded {} usage items", bill.usage_items.len());
        bill
    };

    let mut bill = bill;
    println!("Estimating carbon footprint...");
    estimator::calculate_carbon_impact(&mut bill);

    let output_path = get_env("INPUT_OUTPUT_PATH", "./carbon-estimate.json");

    if let Some(parent) = Path::new(&output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    let json = bill.to_json()?;
    fs::write(&output_path, json)?;

    println!("Carbon estimate saved to: {}", output_path);
    Ok(())
}

fn fetch_billing_usage(token: &str, organization: &str) -> Result<ActionsBill> {
    let url = format!(
        "https://api.github.com/orgs/{}/settings/billing/actions",
        organization
    );

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .context("Failed to send request to GitHub API")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        bail!(
            "Failed to fetch Actions billing: {} - {}",
            status.as_u16(),
            body
        );
    }

    let body = response.text()?;
    ActionsBill::from_json(&body)
}

fn get_env_required(name: &str) -> Result<String> {
    let value = env::var(name).unwrap_or_default();
    if value.is_empty() {
        bail!("Required environment variable not set: {}", name);
    }
    Ok(value)
}

fn get_env(name: &str, default: &str) -> String {
    env::var(name)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}
