<img src="resources/brand/aspen-icon.svg" width="96" alt="aspen">

# Aspen - GitHub Actions Carbon Estimator

[![CI CO2eq](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/DigitalPebble/aspen/badges/badge.json)](https://github.com/DigitalPebble/aspen)
[![CI CO2eq billing](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/DigitalPebble/aspen/badges/badge-billing.json)](https://github.com/DigitalPebble/aspen)

Aspen estimates the energy use and carbon emissions of GitHub Actions for a repository or an organisation. It reads either the jobs of your workflow runs or the GitHub billing report, applies a carbon estimation model (see [Methodology](#methodology)) and writes a JSON report, a markdown summary and an optional badge.

It runs in three ways:

1. [As a GitHub Action with the `jobs` source](#1-github-action-with-the-jobs-source): no secret needed, one repository.
2. [As a GitHub Action with the `billing` source](#2-github-action-with-the-billing-source): needs an organisation token, one repository.
3. [As a CLI outside GitHub Actions](#3-cli-outside-github-actions): the whole organisation.

Aspen estimates emissions from job durations and runner types; it does not measure energy on the runner. If you need measured, per-step energy figures, look at [eco-ci-energy-estimation](https://github.com/green-coding-solutions/eco-ci-energy-estimation/).

> 💡 **Did you know?** Aspens are among the largest living organisms on Earth. A famous quaking aspen grove in Utah called Pando is a single clonal colony of about 47,000 stems. They all share one root system and are genetically identical. It is estimated to weigh around 6,000 tonnes, and parts of the root system may be thousands of years old.

## Jobs vs billing: precision and coverage

| | `jobs` (default) | `billing` |
|---|---|---|
| Minutes | job duration rounded up to whole minutes | as billed |
| Runner type | inferred from the job labels | from the billing SKU |
| Self-hosted and larger runners | reported, but no estimate | as billed |
| Storage | not included | included |
| Per-workflow breakdown | yes, grouped by workflow file | no, the billing API does not expose workflows |
| Period | last `days` days, at most 5,000 runs | last `days` days, or any year, month or day via `--year`, `--month`, `--day` (CLI only) |
| Coverage | the repository the token belongs to | the whole organisation |
| Token | built-in `GITHUB_TOKEN` | organisation `Administration` (read) |
| Badge | ![CI CO2eq](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/DigitalPebble/aspen/badges/badge.json) | ![CI CO2eq billing](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/DigitalPebble/aspen/badges/badge-billing.json) |

`jobs` is an approximation: durations are rounded up and runner types are guessed from labels such as `ubuntu-latest` or `windows-latest`. `billing` is the data GitHub invoices, so it is the more precise of the two and the only one covering storage and the whole organisation, at the cost of a powerful token. Only `jobs` can tell you which workflow the emissions come from.

## 1. GitHub Action with the `jobs` source

Pin the action to a [release](https://github.com/DigitalPebble/aspen/releases) tag. The prebuilt binary is downloaded and its SHA-256 checksum verified, so there is no Rust toolchain and no build step.

```yaml
permissions:
  actions: read

steps:
  - uses: DigitalPebble/aspen@v1.0.0
```

The action analyses the jobs of this repository's workflow runs with the built-in `GITHUB_TOKEN`. The token can only see this repository, and the data it reads is already visible to anyone who can see its Actions runs, so the risk is low.

| Input | Default | Description |
|-------|---------|-------------|
| `source` | `jobs` | `jobs` or `billing` |
| `github_token` | `${{ github.token }}` | With `jobs`, `actions: read` is enough. With `billing`, an organisation token (see step 2) |
| `days` | `30` | Days of usage to analyse, up to today. The same period for both sources, so their figures are comparable |
| `output_path` | `carbon-estimate.json` | Where the report is saved |
| `badge_path` | `badge.json` | Where the badge JSON is written (empty to skip) |
| `publish_badge` | `false` | Commit the badge file to `badge_branch` of this repository (needs `contents: write`) |
| `badge_branch` | `badges` | Branch the badge is published to. Created if missing |
| `version` | the action's tag | Release to download, e.g. `v1.0.0` or `latest` |

| Output | Description |
|--------|-------------|
| `report-path` | Path to the JSON report |
| `badge-path` | Path to the badge JSON |
| `total_co2eq_g` | Total estimated emissions (gCO2eq) |
| `total_energy_wh` | Total estimated energy usage (Wh) |

The action also adds a markdown report (totals, per repository, per workflow and per runner SKU) to the job summary. The per-workflow table is only available with the `jobs` source: workflows are grouped by file, shown by name and linked to the file in the repository. Workflows sharing the same name in different files get separate rows.

The JSON report is written to the runner and discarded when the job ends. To keep it, give the Aspen step an `id` and upload the report as an artifact:

```yaml
steps:
  - uses: DigitalPebble/aspen@v1.0.0
    id: aspen
  - uses: actions/upload-artifact@v4
    with:
      name: carbon-estimate
      path: ${{ steps.aspen.outputs.report-path }}
```

### Add a badge

The action writes a [shields.io endpoint](https://shields.io/badges/endpoint-badge) file with the estimated emissions over the analysed period:

```json
{"schemaVersion":1,"label":"CI CO2eq (30d)","message":"1.20 kgCO2e","color":"c0c0c0"}
```

The colour shows how the data was obtained: **silver** for the `jobs` source (inferred) and **gold** for the `billing` source (as billed).

To display it, let the action publish the file to a branch of your repository. Set `publish_badge: true` and give the job `contents: write`:

```yaml
permissions:
  actions: read
  contents: write

steps:
  - uses: DigitalPebble/aspen@v1.0.0
    with:
      publish_badge: true
```

The file is committed to the `badges` branch (created if it does not exist) as `badge.json`. Use `badge_branch` to pick another branch and `badge_path` to change the file name. Only the file name is kept on the branch, so `badge_path: out/badge-billing.json` is published as `badge-billing.json`. To publish both badges, run the action twice with different `badge_path` values.

Publishing always uses the built-in `GITHUB_TOKEN`, even with `source: billing`, so your organisation token never needs write access. The action works in a temporary directory and leaves your checkout untouched, so no `actions/checkout` step is required.

Then add this to your README, replacing `OWNER/REPO` with your repository (and `badges` / `badge.json` if you changed them):

```markdown
[![CI CO2eq](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/OWNER/REPO/badges/badge.json)](https://github.com/DigitalPebble/aspen)
```

The figures cover a rolling period, so a weekly schedule is usually enough. The badge only works for public repositories, as shields.io cannot read the badge branch of a private one. Keep this workflow restricted like any other with write access.

## 2. GitHub Action with the `billing` source

Set `source: billing` and pass an organisation token:

```yaml
steps:
  - uses: DigitalPebble/aspen@v1.0.0
    with:
      source: billing
      github_token: ${{ secrets.ORG_BILLING_TOKEN }}
```

### Token requirements

Organisation permissions → `Administration` → Read-only. Create a fine-grained personal access token with only this permission. Its resource owner must be your organisation (not your personal account), the organisation may need to approve it, and the token's user must be an organisation owner.

### Security considerations

**Read this before using `billing`, especially in a public repository.**

GitHub only exposes billing data at organisation level, so this token can read billing data for **every repository in your organisation, including private ones**. Aspen mitigates this but cannot remove it:

- Rows for all other repositories are discarded before anything is written to the report, the job summary or the step outputs.
- The filtering happens in Aspen **after** the API call. The token itself is not restricted to one repository.
- Anyone who can modify the workflow or the action reference can run other code with access to the secret and read the whole organisation's billing data. Treat the token as an organisation-wide credential.

Recommendations:

- Pin the action to a release tag or commit SHA, never a branch.
- Store the token in an environment secret limited to protected branches, with required reviewers, rather than a plain repository secret.
- Do not expose the secret to workflows triggered by pull requests from forks, and never use `pull_request_target` with it.
- Restrict who can push to or modify workflows (branch protection, CODEOWNERS on `.github/workflows`).
- Use a dedicated fine-grained token with only `Administration` (read), and rotate it regularly.
- In a public repository where you cannot guarantee the above, use the `jobs` source, run from a private repository, or use the CLI locally.
- The report, job summary and artifacts of a public repository are public. Only the selected repository's data is written to them.

## 3. CLI outside GitHub Actions

The CLI is for administrators who want data for the **entire organisation**. Without `--repository` nothing is filtered, so keep the output private. It needs the same token as the `billing` source.

### Install

Binaries exist for Linux (x86_64, aarch64), macOS (aarch64) and Windows (x86_64). Download the archive for your platform from the [latest release](https://github.com/DigitalPebble/aspen/releases/latest). Assets are named `aspen-<tag>-<target>.tar.gz` (`.zip` on Windows), each with a `.sha256` checksum file. On Linux x86_64:

```bash
curl -LO https://github.com/DigitalPebble/aspen/releases/latest/download/aspen-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
tar xzf aspen-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
```

Put the `aspen` binary somewhere on your `PATH`. To build from source instead, install a [Rust toolchain](https://rustup.rs/) and run:

```bash
git clone https://github.com/DigitalPebble/aspen.git
cd aspen
cargo install --path .
```

### Usage

Fetch the billing report of your organisation and enrich it:

```bash
aspen --token ghp_... --organization your-org
```

By default this covers the last 30 days, like the `jobs` source; change it with `--days`. For a calendar period instead, use `--year`, `--month` and `--day`, e.g. `--year 2026 --month 10` for October.

The output is written to `./carbon-estimate.json`, or to the path given with `--output`. Both values can also come from environment variables (see the options below).

To use the jobs of a single repository instead, with no administrator access:

```bash
aspen --source jobs --repository your-org/your-repo --token ghp_... --days 30
```

To work from a saved billing file, using the [GH CLI](https://cli.github.com/):

```bash
gh api /organizations/$ORG/settings/billing/usage > gh_bill.json
aspen --file gh_bill.json --summary summary.md
```

With no arguments, Aspen reads a billing JSON from stdin and writes the result to stdout. Errors go to stderr.

```bash
gh api /organizations/$ORG/settings/billing/usage | aspen > carbon-estimate.json
```

```
Aspen - GitHub Actions Carbon Estimator

Usage: aspen [OPTIONS]

Options:
  -f, --file <FILE>                    Path to a local billing JSON file (instead of fetching from GitHub API)
  -t, --token <TOKEN>                  GitHub token: `actions: read` on the repository for `jobs`, organization Administration (read) for `billing` [env: INPUT_GITHUB_TOKEN=]
  --source <SOURCE>                    Where to get the usage data from: `billing` (default) or `jobs` [env: INPUT_SOURCE=]
      --days <DAYS>                    Days of usage to analyse, up to today (billing source: ignored when --year, --month or --day is set) [env: INPUT_DAYS=] [default: 30]
      --year <YEAR>                    Year of the billing report (billing source only; replaces --days) [env: INPUT_YEAR=]
      --month <MONTH>                  Month (1-12) of the billing report (billing source only) [env: INPUT_MONTH=]
      --day <DAY>                      Day (1-31) of the billing report (billing source only) [env: INPUT_DAY=]
      --badge <BADGE>                  Path where a shields.io endpoint badge (JSON) will be written [env: INPUT_BADGE_PATH=]
  -r, --repository <REPOSITORY>        Restrict the report to a single repository ("owner/name"); the organization is derived from it [env: INPUT_REPOSITORY=]
  -o, --organization <ORGANIZATION>    GitHub organization name [env: INPUT_ORGANIZATION=]
  -O, --output <OUTPUT>                Path where the carbon estimate report will be saved [env: INPUT_OUTPUT_PATH=] [default: ./carbon-estimate.json]
  -s, --summary <SUMMARY>              File to append a markdown summary to (totals, per repository, per workflow with `jobs`, and per SKU) [env: GITHUB_STEP_SUMMARY=]
  -h, --help                           Print help
  -V, --version                        Print version
```

## Output

Aspen adds two fields to each of the _usageItems_ of the usage report, `energy_usage_wh` and `co2eq_g`:

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

With the `jobs` source, each item also carries `workflow_name` and `workflow_path` (e.g. `.github/workflows/ci.yml`). These keys are absent from `billing` reports.

## Methodology

Energy consumption estimates come from [BoaviztAPI](https://github.com/Boavizta/boaviztapi). The carbon intensity is an average for the US in 2024 from [Ember](https://ember-energy.org). The PUE is as reported by [Microsoft](https://datacenters.microsoft.com/sustainability/efficiency/) for its datacentres in America in 2025.

Storage (`Actions storage`, billed in `GigabyteHours`) uses an SSD energy factor of 1.2 Wh per terabyte hour, with the same PUE and carbon intensity. Items of that SKU with any other unit are ignored.

## Releasing

Run `./release.sh` from a clean checkout (requires the [GH CLI](https://cli.github.com/)). It opens a pull request that sets the version in `Cargo.toml`. Once merged, the script tags `main` with `v<version>`, which triggers the [release workflow](.github/workflows/release.yml). The workflow checks the tag is on `main` and matches `Cargo.toml`, runs clippy and the tests, then creates a **draft** GitHub release with binaries for Linux (x86_64, aarch64), macOS (aarch64) and Windows (x86_64), each with a `.sha256` file.

The release stays a draft until you publish it by hand, so the action cannot download its binaries before then. Once the workflow has finished, open the draft under Releases, check the assets, then tick "Publish this Action to the GitHub Marketplace" (choose a category) and publish. A release has to be published from this page for the Marketplace option to apply, because a second release cannot be created for a tag that already exists.

## Licence

Apache License 2.0

## Credits

Developed by [DigitalPebble](https://digitalpebble.com/).
Please get in touch if you need our help with GreenOps or digital sustainability in general.
You can also sponsor this project, which is always appreciated.

