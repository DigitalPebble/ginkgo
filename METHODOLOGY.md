# Methodology

Aspen estimates the energy use and carbon emissions of GitHub Actions. It does not measure anything on the runner. It takes the minutes a runner was used (or the storage billed), multiplies them by an estimated power draw, adds the data centre overhead and converts the energy to emissions with an average grid carbon intensity.

```
energy (Wh)  = power (W) × minutes / 60 × PUE
co2eq (g)    = energy (Wh) / 1000 × grid carbon intensity (gCO2eq/kWh)
```

## Where the figures live

All the coefficients are in [`resources/config.json`](resources/config.json):

| Key | Value | Unit |
|---|---|---|
| `power_consumption.runners.<runner>.value` | see below | W |
| `storage_energy.ssd.value` | 1.2 | Wh per terabyte hour |
| `grid_carbon_intensity.value` | 383 | gCO2eq/kWh |
| `pue.value` | 1.16 | ratio |

The file is embedded in the binary at compile time (see [`src/config.rs`](src/config.rs)), so changing a value means rebuilding Aspen. They cannot be overridden at run time yet.

The calculation itself is in [`src/estimator.rs`](src/estimator.rs).

## Input data

Aspen works on usage items with the same shape as the GitHub billing report, whichever source they come from.

### `billing` source

The organisation billing report (`/organizations/{org}/settings/billing/usage`) gives the minutes per day, repository and SKU, as GitHub invoices them. The SKU names the runner type directly (`Actions Linux`, `Actions Windows ARM`...). It also includes storage.

### `jobs` source

Aspen lists the completed workflow runs of the repository over the period, then the jobs of each run. For each job:

- The duration is `completed_at - started_at`, **rounded up to whole minutes** as GitHub does for billing. Jobs that never started, or with no duration, are ignored.
- The runner type is **inferred from the job labels** and mapped to a billing SKU (see `sku_for_labels` in [`src/jobs.rs`](src/jobs.rs)):

| Labels contain | SKU |
|---|---|
| `self-hosted` | `Self-hosted` |
| `ubuntu-slim` | `Actions Linux Slim` |
| `ubuntu` or `linux`, plus `arm` | `Actions Linux ARM` |
| `ubuntu` or `linux` | `Actions Linux` |
| `windows`, plus `arm` | `Actions Windows ARM` |
| `windows` | `Actions Windows` |
| `macos` | `Actions macOS` |
| anything else | the first label |

Labels of larger runners are usually custom names, so they end up in the last row.

### Checking the input against GitHub

The minutes Aspen uses with the `jobs` source over the default 30 days should match what GitHub shows in the repository, under **Insights → Actions usage metrics** with the period set to **Last 30 days**.

![Actions usage metrics of the Aspen repository](resources/docs/actions-usage-metrics.png)

For this repository, GitHub reports 117 minutes over 96 job runs. Each job is rounded up to the minute, as Aspen does. The **Runtime OS** column matches the runner types Aspen infers from the labels. GitHub gives a workflow one row per OS, whereas the per-workflow table of Aspen's summary has one row per workflow file, so the three `release.yml` rows here make a single row of 54 minutes in Aspen.

The emissions only cover the minutes Aspen can estimate. In this example the 12 macOS minutes of `release.yml` have no estimate (see [What is not estimated](#what-is-not-estimated)), so the totals cover 105 of the 117 minutes.

## Runner power

Each SKU is mapped to a runner in `config.json`:

| SKU | Runner | Power (W) | Basis |
|---|---|---|---|
| `Actions Linux` | `ubuntu` | 4.315 | 4 vCPU x86 VM, Azure D4as_v4 |
| `Actions Linux Slim` | `ubuntu-slim` | 1.078 | a quarter of `ubuntu` |
| `Actions Linux ARM` | `ubuntu-arm` | 2.157 | 4 vCPU ARM64 VM, Scaleway COPARM1-4C-16G |
| `Actions Windows` | `windows` | 4.746 | `ubuntu` plus 10% for OS overhead |
| `Actions Windows ARM` | `windows-arm` | 2.372 | `ubuntu-arm` plus 10% for OS overhead |

The figures come from [BoaviztAPI](https://github.com/Boavizta/boaviztapi) for these cloud instance types. They are not GitHub's own hardware, which is not published. GitHub's standard hosted runners have had 4 vCPUs since December 2023, hence the choice of 4 vCPU instances.

The power is the instance's draw at **50% CPU load**. The GitHub APIs give no utilisation figures, so Aspen cannot tell a job compiling flat out from one waiting on the network, and applies the same average load to every minute. 50% sits between the two. A server still draws power when idle, so power does not scale linearly with load. A job running close to 100% for its whole duration will be underestimated, and one that is mostly idle overestimated, but by less than the difference in load suggests.

### What is not estimated

Usage on any other SKU gets no `energy_usage_wh` or `co2eq_g` and counts as zero in the totals. This applies to:

- macOS runners (the config has placeholders with a value of `-1`, but no SKU maps to them yet);
- self-hosted runners, whose hardware and location Aspen cannot know;
- larger runners and GPU runners.

These items are still listed in the report, so you can see how many minutes have no estimate.

## Storage

Items of the `Actions storage` SKU billed in `GigabyteHours` (billing source only) use an SSD energy factor of 1.2 Wh per terabyte hour, with the same PUE and carbon intensity:

```
energy (Wh) = 1.2 × gigabyte hours / 1000 × PUE
```

Items of that SKU in any other unit are ignored.

## PUE

The Power Usage Effectiveness accounts for cooling, power distribution and other data centre overhead. Aspen uses 1.16, the value [reported by Microsoft](https://datacenters.microsoft.com/sustainability/efficiency/) for its data centres in the Americas in 2025, since GitHub hosted runners run on Azure.

## Grid carbon intensity

Aspen uses 383 gCO2eq/kWh, the average for the US in 2024 from [Ember](https://ember-energy.org). GitHub does not say which Azure regions its runners use, but most of them are in the US. The figure is a yearly average and does not reflect the time of day a job ran or the renewable energy purchased by Microsoft (location based, not market based).

## Worked example

A 10 minute job on `ubuntu-latest`:

```
energy = 4.315 W × 10 / 60 × 1.16 = 0.834 Wh
co2eq  = 0.834 / 1000 × 383       = 0.320 g
```

1,000 gigabyte hours of Actions storage:

```
energy = 1.2 × 1000 / 1000 × 1.16 = 1.392 Wh
co2eq  = 1.392 / 1000 × 383       = 0.533 g
```

## Outputs

Every usage item that could be estimated gets two fields, `energy_usage_wh` and `co2eq_g`, rounded to three decimals. The markdown summary adds them up per repository, per workflow (`jobs` source only) and per SKU.

The badge shows the total over the period, in g or kg CO2eq. Its colour shows where the data came from: silver (`c0c0c0`) for the `jobs` source, where minutes and runner types are inferred, and gold for the `billing` source, where they are as billed.

## Limitations

- **Embodied emissions are not included.** Only the electricity used while the runner is busy is counted, not the emissions from manufacturing the hardware.
- **Network and artifact transfer are not included.**
- **CPU load is not known.** Every minute of a given runner type is priced at 50% load (see [Runner power](#runner-power)), whatever the job was actually doing.
- **The `jobs` source is an approximation.** Durations are rounded up per job and runner types are guessed from labels. The source covers at most 5,000 runs.
- **One grid and one PUE for everything.** All jobs are treated as running in the US, at the same time of year.

If you have better figures for any of these, issues and pull requests are welcome.
