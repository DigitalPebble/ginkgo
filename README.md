# 🌳 Ginkgo - GitHub Actions Carbon Estimator

Estimate the environmental impact of GitHub Actions for your entire organization.

## About

Ginkgo helps organizations understand and track the carbon footprint of their GitHub Actions workflows by analyzing billing data and applying carbon estimation models.

Named after the ancient Ginkgo tree, one of the world's oldest living tree species and a symbol of resilience and sustainability.

## Features

- Fetches GitHub Actions billing data (minutes used across all runners)
- Calculates estimated carbon emissions based on compute usage
- Supports Packages and Storage billing analysis
- Generates detailed reports for organizational tracking
- Runs as a GitHub Action in your workflows

## Usage

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

## Inputs

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

**Current implementation is a placeholder.** You'll need to replace it with:
- Runner-specific energy consumption models (different runners have different power profiles)
- Regional carbon intensity data (data center locations have different grid carbon intensity)
- Time-based analysis (grid carbon intensity varies by time of day)
- Storage and network transfer estimates

### Recommended Resources for Implementation

- [Green Software Foundation Carbon Aware SDK](https://github.com/Green-Software-Foundation/carbon-aware-sdk)
- [Cloud Carbon Footprint](https://www.cloudcarbonfootprint.org/)
- [GitHub's own sustainability approach](https://github.blog/changelog/2021-04-22-github-actions-all-actions-will-run-on-node16-instead-of-node12/)

## Development

### Building Locally

```bash
mvn clean package
```

### Testing the Docker Image

```bash
docker build -t ginkgo .
docker run --rm \
  -e INPUT_GITHUB-TOKEN=your_token \
  -e INPUT_ORGANIZATION=your_org \
  -e INPUT_OUTPUT-PATH=carbon-estimate.json \
  -v $(pwd):/github/workspace \
  ginkgo
```

## Contributing

Contributions are welcome, especially:
- Improved carbon estimation models
- Support for additional runner types
- Regional carbon intensity data integration
- Visualization and reporting enhancements

## License

Apache License 2.0

## Credits

Developed by [DigitalPebble](https://digitalpebble.com/)
