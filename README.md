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

## Usage

### GitHub action

** WORK IN PROGRESS **

```yaml
name: Carbon Footprint Report

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday
  workflow_dispatch:

jobs:
  estimate-carbon:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      
      - name: Estimate Carbon Footprint
        uses: digitalpebble/ginkgo@v1
        with:
          github-token: ${{ secrets.BILLING_TOKEN }}
          organization: your-org-name
          output-path: reports/carbon-estimate.json
      
      - name: Upload Report
        uses: actions/upload-artifact@v4
        with:
          name: carbon-estimate
          path: reports/carbon-estimate.json
```

### Running Locally with a Billing File

Instead of fetching data from the GitHub API, you can pass a local billing JSON file as an argument:

```bash
java -jar target/ginkgo.jar actions_bill.json
```

The output path defaults to `carbon-estimate.json` and can be overridden with the `OUTPUT-PATH` environment variable.

## Inputs

When running as a CLI tool, the first argument is an optional path to a local billing JSON file. If provided, the GitHub API is not called and `github-token`/`organization` inputs are not required.

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `github-token` | Yes | - | GitHub token with `billing:read` permission |
| `organization` | Yes | - | GitHub organization name |
| `output-path` | No | `carbon-estimate.json` | Path where the report will be saved |

## Outputs

| Output | Description |
|--------|-------------|
| `report-path` | Path to the generated carbon estimate report |

## Required Permissions

The GitHub token must have:
- `billing:read` - To access organization billing information

You can create a fine-grained personal access token with only the `billing:read` permission scoped to your organization.

## Carbon Estimation Methodology

The carbon estimation logic is implemented in the `calculateCarbonImpact` method in `CarbonEstimator.java`. 

The energy consumption estimates are taken from [BoaviztAPI](https://github.com/Boavizta/boaviztapi), the carbon intensity factor
is an average for the Azure regions from the [ElectricityMaps](https://app.electricitymaps.com/datasets) datasets for 2024. 
The PUE is as reported by [Microsoft](https://datacenters.microsoft.com/sustainability/efficiency/) for the datacentres in America in 2025. 

## Development

### Building Locally

```bash
mvn clean package
```

### Testing the Docker Image

```bash
mkdir output
docker build -t ginkgo .
docker run --rm \
  -e INPUT_GITHUB-TOKEN=your_token \
  -e INPUT_ORGANIZATION=your_org \
  -e OUTPUT-PATH=/ginkgo/carbon-estimate.json \
  -v $(pwd)/output:/ginkgo \
  ginkgo
```

## License

Apache License 2.0

## Credits

Developed by [DigitalPebble](https://digitalpebble.com/). Please get in touch if you need our help with GreenOps or digital sustainability in general.
