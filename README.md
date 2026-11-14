# Ginkgo - GitHub Actions Carbon Estimator

Estimate the environmental impact of GitHub Actions for your entire organization.

## About

Ginkgo helps organizations understand and track the carbon footprint of their GitHub Actions workflows by analyzing billing data and applying carbon estimation models.
For measuring carbon emissions per workflow, look at [eco-ci-energy-estimation](https://github.com/green-coding-solutions/eco-ci-energy-estimation/).
The billing data can be retrieved using the [GitHub API](https://docs.github.com/en/rest/billing/usage?apiVersion=2022-11-28#get-billing-usage-report-for-an-organization).

## Features

- Calculates estimated carbon emissions for GitHub Action based on billed usage
- Fetches GitHub Actions billing data with a token and enriches it
- Runs as a GitHub Action in your workflows or on the command line

## Output

Ginkgo takes as input a JSON usage report and enriches it by adding two fields to the _usageItems_:
* `energy_usage_wh`
* `co2eq_g`

Here is what an enriched _usageItem_ looks like

```json
{
  "date": "2026-02-01T04:48:44Z",
  "product": "actions",
  "sku": "Actions Linux",
  "quantity": 91,
  "unitType": "Minutes",
  "pricePerUnit": 0.006,
  "grossAmount": 0.546,
  "discountAmount": 0.546,
  "netAmount": 0,
  "organizationName": "DigitalPebble",
  "repositoryName": "spruce",
  "energy_usage_wh": 7.592,
  "co2eq_g": 2.672
}
```

## Installation

### Prebuilt binaries (recommended)

Download the archive for your platform from the [latest release](https://github.com/DigitalPebble/ginkgo/releases/latest). Binaries are available for Linux (x86_64, aarch64), macOS (aarch64) and Windows (x86_64).

Assets are named `ginkgo-<tag>-<target>.tar.gz` (`.zip` on Windows), with a matching `.sha256` checksum file. For example on Linux x86_64:

```bash
curl -LO https://github.com/DigitalPebble/ginkgo/releases/latest/download/ginkgo-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz
tar xzf ginkgo-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz
```

Replace `vX.Y.Z` with the tag of the release, then put the `ginkgo` binary somewhere on your `PATH`.

### Build from source

As an alternative, or if no binary exists for your platform, build Ginkgo yourself. Prerequisite: install a [Rust toolchain](https://rustup.rs/).

```bash
git clone https://github.com/DigitalPebble/ginkgo.git
cd ginkgo
cargo install --path .
```

This installs `ginkgo` into `~/.cargo/bin`. To only build it, run `cargo build --release`; the binary is then `target/release/ginkgo`.

## CLI Usage

```
Ginkgo - GitHub Actions Carbon Estimator

Usage: ginkgo [OPTIONS]

Options:
  -f, --file <FILE>                    Path to a local billing JSON file (instead of fetching from GitHub API)
  -t, --token <TOKEN>                  GitHub token with organization Administration (read) permission [env: INPUT_GITHUB_TOKEN=]
  -o, --organization <ORGANIZATION>    GitHub organization name [env: INPUT_ORGANIZATION=]
  -O, --output <OUTPUT>                Path where the carbon estimate report will be saved [env: INPUT_OUTPUT_PATH=] [default: ./carbon-estimate.json]
  -s, --summary <SUMMARY>              File to append a markdown summary to (totals, per repository and per SKU) [env: GITHUB_STEP_SUMMARY=]
  -h, --help                           Print help
  -V, --version                        Print version
```

### Markdown summary

Use `--summary` to also get a markdown report (totals, per repository and per runner SKU), handy outside of GitHub Actions too:

```bash
ginkgo --file gh_bill.json --summary summary.md
```

The report is appended to the file, which is created if needed.

### From a local billing file

With the [GH CLI](https://cli.github.com/) installed:

```bash
ORG=enter_org_here
gh api /organizations/$ORG/settings/billing/usage > gh_bill.json
ginkgo --file gh_bill.json
```

This will produce an enriched version of the usage reports in `./carbon-estimate.json`. You can specify a different output with `-O output_file`.

### As a filter (stdin → stdout)

When called with no arguments, Ginkgo reads a billing JSON from stdin and writes the enriched result to stdout, making it easy to compose with other tools:

```bash
ORG=enter_org_here
gh api /organizations/$ORG/settings/billing/usage | ginkgo
```

To save it to a file:

```bash
gh api /organizations/$ORG/settings/billing/usage | ginkgo > carbon-estimate.json
```

Error messages are written to stderr; stdout contains only the JSON output.

### Retrieve and enrich the usage reports from the GitHub API

You need to specify both the name of your organization and a token. 
This can be done on the command line or as an environment variable. See required permissions below.

```bash
ginkgo --token ghp_... --organization your-org
```

The output is written to the path specified by `--output` (defaults to `./carbon-estimate.json`).

## GitHub Action Usage

Pin the action to a [release](https://github.com/DigitalPebble/ginkgo/releases) tag: the matching prebuilt binary is downloaded and its SHA-256 checksum verified, so there is no Rust toolchain to install and no build step.

```yaml
- uses: DigitalPebble/ginkgo@vX.Y.Z
  with:
    github_token: ${{ secrets.BILLING_TOKEN }}
    organization: your-org
    output_path: carbon-estimate.json  # optional
```

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `github_token` | Yes | - | GitHub token with organization `Administration` (read) permission |
| `organization` | Yes | - | GitHub organization name |
| `output_path` | No | `carbon-estimate.json` | Path where the report will be saved |
| `version` | No | the action's tag | Release tag to download, e.g. `v0.3.0` or `latest` |

Binaries are available for Linux (x86_64, aarch64), macOS (aarch64) and Windows (x86_64). On other runners, or when the action is used from a branch or commit (`@main`, `uses: ./`), it is built from source instead, which is slower.

| Output | Description |
|--------|-------------|
| `report-path` | Path to the generated JSON report |
| `total_co2eq_g` | Total estimated emissions (gCO2eq) |
| `total_energy_wh` | Total estimated energy usage (Wh) |

The action also adds a markdown report (totals, per repository and per runner SKU) to the job summary.
The CLI does the same whenever the runner's `GITHUB_STEP_SUMMARY` / `GITHUB_OUTPUT` environment variables are set. `GITHUB_OUTPUT` is read from the environment only; it has no documented flag.

The built-in `GITHUB_TOKEN` cannot read organization billing: store a token with billing access in a secret.

## Required Permissions

The GitHub token must have:
- Organization permissions → `Administration` → Read-only, to access organization billing information

You can create a fine-grained personal access token with only this permission. Its resource owner must be your organization (not your personal account), and the organization may need to approve it. The token's user must be an organization owner.

## Carbon Estimation Methodology

The energy consumption estimates are taken from [BoaviztAPI](https://github.com/Boavizta/boaviztapi), the carbon intensity factor
is an average for the US in 2024 from [Ember](https://ember-energy.org).
The PUE is as reported by [Microsoft](https://datacenters.microsoft.com/sustainability/efficiency/) for the datacentres in America in 2025.

Storage (`Actions storage`, billed in `GigabyteHours`) is estimated from an SSD energy factor of 1.2 Wh per terabyte-hour, with the same PUE and carbon intensity applied. Items of that SKU with any other unit are ignored.

## Releasing

Run `./release.sh` from a clean checkout (requires the [GH CLI](https://cli.github.com/)). It opens a pull request that sets the version in `Cargo.toml`, so `main` can stay protected. Once the PR is merged, the script tags `main` with `v<version>` (for example `v0.3.0`), which triggers the [release workflow](.github/workflows/release.yml). The workflow checks the tag is on `main` and matches `Cargo.toml`, runs clippy and the tests, then publishes a GitHub release with binaries for Linux (x86_64, aarch64), macOS (aarch64) and Windows (x86_64), each with a `.sha256` checksum file.

## License

Apache License 2.0

## Credits

Developed by [DigitalPebble](https://digitalpebble.com/). Please get in touch if you need our help with GreenOps or digital sustainability in general.
