# cpg

CpG island analysis toolkit written in Rust. Accepts FASTA input and provides
three subcommands: island detection, sequence statistics, and sliding-window profiling.

## Install

### Install via cargo (You must have Rust installed)

```bash
cargo install cpg-bio
```

### Precompiled binary

Download the binary for your system from the
[latest release](https://github.com/marceelrf/cpg/releases/latest):

**Linux**
```bash
curl -LO https://github.com/marceelrf/cpg/releases/latest/download/cpg-linux-x86_64
chmod +x cpg-linux-x86_64
mv cpg-linux-x86_64 ~/.local/bin/cpg
```

**macOS**
```bash
curl -LO https://github.com/marceelrf/cpg/releases/latest/download/cpg-macos-aarch64
chmod +x cpg-macos-aarch64
mv cpg-macos-aarch64 ~/.local/bin/cpg
```

**Windows** — download `cpg-windows-x86_64.exe` from the releases page.


## Subcommands

### `finder` — detect CpG islands

```bash
# Default: Gardiner-Garden & Frommer 1987, 4 threads
cpg finder -i genome.fa

# Takai & Jones 2002
cpg finder -i genome.fa --criteria takai

# Fully custom thresholds
cpg finder -i genome.fa --min-length 300 --min-gc 0.52 --min-obs-exp 0.62

# Mix: start from a preset and override one threshold
cpg finder -i genome.fa --criteria takai --min-length 300

# BED output — open directly in IGV or bedtools
cpg finder -i genome.fa --output bed > islands.bed

# Filter results to a specific genomic region
cpg finder -i genome.fa --region chr1:1000000-2000000

# Use the RunningSum algorithm (CpGcluster-style)
cpg finder -i genome.fa --algorithm runningsum

# Combine: RunningSum + region filter + BED output
cpg finder -i genome.fa --algorithm runningsum --region chr1:1000000-2000000 --output bed
```

Built-in presets:

| Preset     | Length | GC%  | Obs/Exp |
|------------|--------|------|---------|
| `gardiner` | ≥ 200  | ≥ 50 | ≥ 0.60  |
| `takai`    | ≥ 500  | ≥ 55 | ≥ 0.65  |

#### `--region` flag

Restricts output to islands that overlap a given genomic interval.
Format: `chrom:start-end` (0-based, same convention as BED).

```bash
cpg finder -i genome.fa --region chr1:1000000-2000000
cpg finder -i genome.fa -r chr1:1000000-2000000          # short form
```

The chromosome name must match the sequence ID in the FASTA header exactly.
No index (`.fai`) is required — the full FASTA is read and results are
filtered after detection.

#### `--algorithm` flag

| Algorithm      | Description |
|----------------|-------------|
| `slidingwindow` | *(default)* Seed-and-extend: scans the sequence in windows of `min_length`, extends any seed that meets GC and Obs/Exp thresholds, and merges overlapping islands. |
| `runningsum`    | CpGcluster-style scoring (Hackenberg et al. 2006): each CpG dinucleotide adds +17 to a running score; each non-CpG base subtracts 1. An island opens when the score exceeds 0 and closes when it returns to 0. Results are filtered by the active criteria. |

Both algorithms respect all `--criteria` / `--min-*` flags and all output formats.

### `stats` — per-sequence statistics

```bash
cpg stats -i sequences.fa
cpg stats -i sequences.fa --output tsv
cpg stats -i sequences.fa --output json
```

Outputs: `ID`, `Length`, `CG_count`, `GC%`, `Obs/Exp`.

### `profiler` — sliding-window metric profile

```bash
# Obs/Exp in 200 bp windows stepping 50 bp (default)
cpg profiler -i genome.fa --window 200 --step 50 --metric obs_exp

# GC% — TSV output, easy to pipe into R or Python
cpg profiler -i genome.fa -w 100 -s 10 -m gc_percent --output tsv

# BEDGRAPH output — load directly into IGV or UCSC Genome Browser
cpg profiler -i genome.fa -w 200 -s 50 -m obs_exp --output bedgraph > profile.bedgraph
```

Metrics: `cg_count`, `gc_percent`, `obs_exp`.

## Global flags

These flags work with every subcommand:

| Flag        | Default | Description                              |
|-------------|---------|------------------------------------------|
| `--output`  | `table` | Output format (see table below)          |
| `--threads` | `4`     | Number of threads (`0` = all available)  |

## Output formats

| Format      | Available in       | Use case                          |
|-------------|-------------------|-----------------------------------|
| `table`     | all               | terminal / quick inspection       |
| `tsv`       | all               | R, Python, Excel                  |
| `json`      | all               | downstream scripts                |
| `bed`       | finder            | IGV, bedtools, UCSC               |
| `bedgraph`  | profiler          | IGV, UCSC Genome Browser          |


# Funding

We thank [FAPESP](https://fapesp.br/)(2025/28063-3) and [CAPES](http://capes.gov.br/)(PIPD 88887.114278/2025-00) for the financial support.
