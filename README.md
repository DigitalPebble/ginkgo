# Ginkgo - GitHub Actions Carbon Estimator

Estimate the environmental impact of GitHub Actions for a repository, or for a whole organization.

## About

Ginkgo helps organizations understand and track the carbon footprint of their GitHub Actions workflows by analyzing billing data and applying carbon estimation models.
For measuring carbon emissions per workflow, look at [eco-ci-energy-estimation](https://github.com/green-coding-solutions/eco-ci-energy-estimation/).
The billing data can be retrieved using the [GitHub API](https://docs.github.com/en/rest/billing/usage?apiVersion=2022-11-28#get-billing-usage-report-for-an-organization).

## Features

- Calculates estimated carbon emissions for GitHub Action based on billed usage
- Fetches GitHub Actions billing data with a token and enriches it
- Runs as a GitHub Action scoped to the repository it is used in (most users), with no secret needed
- Runs on the command line for organization-wide reports (administrators)

## GitHub Action Usage (per repository)

Pin the action to a [release](https://github.com/DigitalPebble/ginkgo/releases) tag: the matching prebuilt binary is downloaded and its SHA-256 checksum verified, so there is no Rust toolchain to install and no build step.

```yaml
permissions:
  actions: read

steps:
  - uses: DigitalPebble/ginkgo@vX.Y.Z
```

That is all: by default the action analyses the jobs of this repository's workflow runs with the built-in `GITHUB_TOKEN`. No secret and no administrator access are needed.

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `source` | No | `jobs` | `jobs` (workflow jobs of this repository) or `billing` (organization billing API, filtered to this repository) |
| `github_token` | No | `${{ github.token }}` | With `jobs`: `actions: read` is enough. With `billing`: a token with organization `Administration` (read) permission |
| `days` | No | `30` | Days of workflow runs to analyse (`jobs` source only) |
| `output_path` | No | `carbon-estimate.json` | Path where the report will be saved |
| `badge_path` | No | `badge.json` | Path where the [badge](#badge) JSON will be written (empty to skip) |
| `version` | No | the action's tag | Release tag to download, e.g. `v0.3.0` or `latest` |

Binaries are available for Linux (x86_64, aarch64), macOS (aarch64) and Windows (x86_64). On other runners, or when the action is used from a branch or commit (`@main`, `uses: ./`), it is built from source instead, which is slower.

| Output | Description |
|--------|-------------|
| `report-path` | Path to the generated JSON report |
| `badge-path` | Path to the generated badge JSON |
| `total_co2eq_g` | Total estimated emissions (gCO2eq) |
| `total_energy_wh` | Total estimated energy usage (Wh) |

The action also adds a markdown report (totals, per repository and per runner SKU) to the job summary.
The CLI does the same whenever the runner's `GITHUB_STEP_SUMMARY` / `GITHUB_OUTPUT` environment variables are set. `GITHUB_OUTPUT` is read from the environment only; it has no documented flag.

The action only ever reports on the repository it runs in (`github.repository`). To report on a whole organization, use the CLI.

### Badge

The action writes a [shields.io endpoint](https://shields.io/badges/endpoint-badge) file (`badge.json` by default) with the estimated emissions over the analysed period:

```json
{"schemaVersion":1,"label":"CI CO2eq (30d)","message":"1.20 kgCO2e","color":"c0c0c0"}
```

The colour reflects how the data was obtained: **silver** with the `jobs` source (no key, runner types are inferred, less accurate) and **gold** with the `billing` source (key with organization access, data as billed). The label includes the period for the `jobs` source and for a `billing` report restricted with `--year`/`--month`/`--day`.

To display it, publish the file at a public URL and point shields.io to it. The simplest way is a `badges` branch, which only needs the built-in token with `contents: write` (no extra secret):

```yaml
permissions:
  actions: read
  contents: write

steps:
  - uses: actions/checkout@v4
  - uses: DigitalPebble/ginkgo@vX.Y.Z
    id: ginkgo
  - name: Publish badge
    env:
      BADGE_PATH: ${{ steps.ginkgo.outputs.badge-path }}
    run: |
      set -euo pipefail
      cp "$BADGE_PATH" "$RUNNER_TEMP/badge.json"
      git config user.name "github-actions[bot]"
      git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
      if git ls-remote --exit-code --heads origin badges >/dev/null; then
        git fetch --depth=1 origin badges
        git checkout -f -B badges FETCH_HEAD
      else
        git checkout --orphan badges
        git rm -rf --quiet .
      fi
      cp "$RUNNER_TEMP/badge.json" badge.json
      git add badge.json
      git diff --cached --quiet || git commit -m "Update carbon badge"
      git push origin badges
```

Then add this to your README (replace `OWNER/REPO`):

```markdown
![CI CO2eq](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/OWNER/REPO/badges/badge.json)
```

The figures are estimates and cover a rolling period, so the badge changes each time the workflow runs: a weekly schedule is usually enough. The badge only works for public repositories, as shields.io cannot read the `badges` branch of a private one. `contents: write` lets the job push to the repository, so keep this workflow restricted like any other that holds write access, and use `jobs` rather than `billing` unless you need the billed data.

### Jobs source vs billing source

| | `jobs` (default) | `billing` |
|---|---|---|
| Token | built-in `GITHUB_TOKEN`, `actions: read` | secret with organization `Administration` (read) |
| Can the token see other repositories? | No | **Yes, the whole organization** |
| Minutes | job duration rounded up to whole minutes | as billed |
| Runner type | inferred from the job's labels | from the billing SKU |
| Storage usage | not available | included |
| API usage | one request per workflow run (the `GITHUB_TOKEN` limit is 1,000 requests per hour per repository) | one request |

With the `jobs` source, a runner type is inferred from the labels (`ubuntu-latest`, `ubuntu-24.04-arm`, `windows-latest`...). Self-hosted runners and custom or larger runner labels appear in the report under their own SKU but get no estimate. Only the last `days` days of completed runs are read, and at most 5,000 runs.

## Security considerations

**Read this before using the action, especially in a public repository.**

### Default (`jobs` source): low risk

The built-in `GITHUB_TOKEN` can only access the repository the workflow runs in, and the job data it reads is already visible to anyone who can see that repository's Actions runs. Nothing about other repositories can leak, even from a public repository. Grant only `permissions: actions: read`.

### `billing` source: organization-wide credential

GitHub only exposes billing data at organization level, so the token this source needs (organization `Administration` read) can read billing data for **every repository in your organization, including private ones**. Ginkgo mitigates this but cannot remove it:

- Rows for all repositories other than the current one are discarded before anything is written to the report, the job summary or the step outputs.
- This filtering is done by Ginkgo **after** the API call. The token itself is not restricted to one repository.
- Anyone who can modify the workflow (or the action reference) in that repository can make it run other code with access to the secret, and read the whole organization's billing data. Treat the token as an organization-wide credential.

Recommendations when using `billing`:

- Pin the action to a release tag or commit SHA, never a branch.
- Store the token in an environment secret limited to protected branches, with required reviewers, rather than a plain repository secret.
- Do not expose the secret to workflows triggered by pull requests from forks, and never use `pull_request_target` with it.
- Restrict who can push to or modify workflows in the repository (branch protection, CODEOWNERS on `.github/workflows`).
- Use a dedicated fine-grained token with only `Administration` (read), and rotate it regularly.
- If the repository is public and you cannot guarantee the above, do not use `billing` there: use the default `jobs` source, run it from a private repository, or use the CLI locally.
- The report, job summary and artifacts of a public repository are public: only the selected repository's data is written to them.

## Required Permissions

- `jobs` source: the workflow needs `permissions: actions: read`. With the CLI, a token with `Actions` → Read-only on the repository (fine-grained) or the `repo` scope.
- `billing` source and organization-wide CLI use: Organization permissions → `Administration` → Read-only, to access organization billing information. You can create a fine-grained personal access token with only this permission. Its resource owner must be your organization (not your personal account), and the organization may need to approve it. The token's user must be an organization owner.

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

## CLI Usage (organization-wide, for administrators)

The CLI is meant for administrators running Ginkgo outside of a repository to get data for the **entire organization**. Without `--repository` nothing is filtered, so keep the output private.

```
Ginkgo - GitHub Actions Carbon Estimator

Usage: ginkgo [OPTIONS]

Options:
  -f, --file <FILE>                    Path to a local billing JSON file (instead of fetching from GitHub API)
  -t, --token <TOKEN>                  GitHub token with organization Administration (read) permission [env: INPUT_GITHUB_TOKEN=]
  --source <SOURCE>                    Where to get the usage data from: `billing` (default) or `jobs` [env: INPUT_SOURCE=]
      --days <DAYS>                    Days of workflow runs to analyse (jobs source only) [env: INPUT_DAYS=] [default: 30]
      --year <YEAR>                    Year of the billing report (billing source only) [env: INPUT_YEAR=]
      --month <MONTH>                  Month (1-12) of the billing report (billing source only) [env: INPUT_MONTH=]
      --day <DAY>                      Day (1-31) of the billing report (billing source only) [env: INPUT_DAY=]
      --badge <BADGE>                  Path where a shields.io endpoint badge (JSON) will be written [env: INPUT_BADGE_PATH=]
  -r, --repository <REPOSITORY>        Restrict the report to a single repository ("owner/name"); the organization is derived from it [env: INPUT_REPOSITORY=]
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

To analyse a single repository from its workflow jobs instead of the billing API (no administrator access needed):

```bash
ginkgo --source jobs --repository your-org/your-repo --token ghp_... --days 30
```

The output is written to the path specified by `--output` (defaults to `./carbon-estimate.json`).

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
