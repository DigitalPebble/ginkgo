# 🌳 Ginkgo - GitHub Actions Carbon Estimator

Estimate the environmental impact of GitHub Actions for your entire organization.

## About

Ginkgo helps organizations understand and track the carbon footprint of their GitHub Actions workflows by analyzing billing data and applying carbon estimation models.
For measuring carbon emissions per workflow, look at [CarbonCi](https://github.com/green-coding-solutions/eco-ci-energy-estimation/).
The billing data can be retrieved using the [GitHub API](https://docs.github.com/en/rest/billing/usage?apiVersion=2022-11-28).

Named after the ancient Ginkgo tree, one of the world's oldest living tree species and a symbol of resilience and sustainability.

## Features

- Fetches GitHub Actions billing data (minutes used across all runners)
- Calculates estimated carbon emissions based on billed usage
- Runs as a GitHub Action in your workflows or on the command line 

## Build

```bash
cargo build --release
```

## Usage

### From a local billing file

With the *GH CLI* and *jq* installed

```bash
export GH_ORG=your-org

gh api /organizations/$GH_ORG/settings/billing/usage | jq > gh_bill.json 
./target/release/ginkgo gh_bill.json 
```

## Inputs

When running as a CLI tool, the first argument is an optional path to a local billing JSON file. If provided, the GitHub API is not called and `github-token`/`organization` inputs are not required.

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `github-token` | Yes | - | GitHub token with `billing:read` permission |
| `organization` | Yes | - | GitHub organization name |
| `output-path` | No | `carbon-estimate.json` | Path where the report will be saved |


### From the GitHub API (WORK IN PROGRESS)

Set the required environment variables and run without arguments:

```bash
export GH_TOKEN=ghp_...
export GH_ORG=your-org

./target/release/ginkgo
```

The output is written to the path specified by the `OUTPUT_PATH` environment variable (defaults to `./carbon-estimate.json`).

## Required Permissions

The GitHub token must have:
- `billing:read` - To access organization billing information

You can create a fine-grained personal access token with only the `billing:read` permission scoped to your organization.

## Carbon Estimation Methodology

The carbon estimation logic is implemented in the `calculateCarbonImpact` method in `CarbonEstimator.java`. 

The energy consumption estimates are taken from [BoaviztAPI](https://github.com/Boavizta/boaviztapi), the carbon intensity factor
is an average for the Azure regions from the [ElectricityMaps](https://app.electricitymaps.com/datasets) datasets for 2024. 
The PUE is as reported by [Microsoft](https://datacenters.microsoft.com/sustainability/efficiency/) for the datacentres in America in 2025. 


## License

Apache License 2.0

## Credits

Developed by [DigitalPebble](https://digitalpebble.com/). Please get in touch if you need our help with GreenOps or digital sustainability in general.
```
