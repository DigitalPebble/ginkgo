# 🌳 Ginkgo - GitHub Actions Carbon Estimator

Estimate the environmental impact of GitHub Actions for your entire organization.

## About

Ginkgo helps organizations understand and track the carbon footprint of their GitHub Actions workflows by analyzing billing data and applying carbon estimation models.
For measuring carbon emissions per workflow, look at [CarbonCi](https://github.com/green-coding-solutions/eco-ci-energy-estimation/).
The billing data can be retrieved using the [GitHub API](https://docs.github.com/en/rest/billing/usage?apiVersion=2022-11-28#get-billing-usage-report-for-an-organization).

## Features

- Calculates estimated carbon emissions for GitHub Action based on billed usage
- ~~Fetches GitHub Actions billing data~~ NOT YET
- ~~Runs as a GitHub Action in your workflows or on the command line~~ NOT YET

## Build

Prerequisite: install a [Rust toolchain](https://rustup.rs/) locally.

```bash
cargo install --path .
```

## CLI Usage

```
Ginkgo - GitHub Actions Carbon Estimator

Usage: ginkgo [OPTIONS]

Options:
  -f, --file <FILE>                  Path to a local billing JSON file (instead of fetching from GitHub API)
  -t, --token <TOKEN>                GitHub token with billing:read permission [env: INPUT_GITHUB_TOKEN]
  -o, --organization <ORGANIZATION>  GitHub organization name [env: INPUT_ORGANIZATION]
  -O, --output <OUTPUT>              Path where the carbon estimate report will be saved [env: INPUT_OUTPUT_PATH] [default: ./carbon-estimate.json]
  -h, --help                         Print help
  -V, --version                      Print version
```

### From a local billing file

With the [GH CLI](https://cli.github.com/) and [jq](https://jqlang.org/) installed:

```bash
gh api /organizations/your-org/settings/billing/usage | jq > gh_bill.json
ginkgo --file gh_bill.json
```

### Retrieve and enrich the usage reports from the GitHub API

**WORK IN PROGRESS** you need to specify both the name of your organization and a token. 
This can be done on the command line or as an environment variable.

```bash
ginkgo --token ghp_... --organization your-org
```

The output is written to the path specified by `--output` (defaults to `./carbon-estimate.json`).

## GitHub Action Usage

**WORK IN PROGRESS** 

```yaml
- uses: DigitalPebble/ginkgo@main
  with:
    github_token: ${{ secrets.BILLING_TOKEN }}
    organization: your-org
    output_path: carbon-estimate.json  # optional
```

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `github_token` | Yes | - | GitHub token with `billing:read` permission |
| `organization` | Yes | - | GitHub organization name |
| `output_path` | No | `carbon-estimate.json` | Path where the report will be saved |

## Required Permissions

The GitHub token must have:
- `billing:read` - To access organization billing information

You can create a fine-grained personal access token with only the `billing:read` permission scoped to your organization.

## Carbon Estimation Methodology

The energy consumption estimates are taken from [BoaviztAPI](https://github.com/Boavizta/boaviztapi), the carbon intensity factor
is an average for the Azure regions from the [ElectricityMaps](https://app.electricitymaps.com/datasets) datasets for 2024.
The PUE is as reported by [Microsoft](https://datacenters.microsoft.com/sustainability/efficiency/) for the datacentres in America in 2025.

## License

Apache License 2.0

## Credits

Developed by [DigitalPebble](https://digitalpebble.com/). Please get in touch if you need our help with GreenOps or digital sustainability in general.
