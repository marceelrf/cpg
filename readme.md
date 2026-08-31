# cpg

CpG island analysis toolkit written in Rust. Accepts FASTA input and provides
three subcommands: island detection, sequence statistics, and sliding-window profiling.

## Install

<!--
```bash
# Soon
# cargo install --path .
```
-->


Download the binary for your system from the
[latest release](https://github.com/marceelrf/cpg/releases/latest):

### Linux / macOS
```bash
curl -LO https://github.com/marceelrf/cpg/releases/download/v0.1.0/cpg-linux-x86_64  
chmod +x cpg-linux-x86_64  
mv cpg-linux-x86_64 ~/.local/bin/cpg  
```

## Subcommands

### `finder` — detect CpG islands

```bash
# Built-in preset (default: Gardiner-Garden & Frommer 1987)
cpg finder -i genome.fa

# Takai & Jones 2002
cpg finder -i genome.fa --criteria takai

# Fully custom thresholds
cpg finder -i genome.fa --min-length 300 --min-gc 0.52 --min-obs-exp 0.62

# Mix: start from a preset and override one threshold
cpg finder -i genome.fa --criteria takai --min-length 300
```

Built-in presets:

| Preset     | Length | GC%  | Obs/Exp |
|------------|--------|------|---------|
| `gardiner` | ≥ 200  | ≥ 50 | ≥ 0.60  |
| `takai`    | ≥ 500  | ≥ 55 | ≥ 0.65  |

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

# GC% in 100 bp windows, TSV output (easy to pipe into R/Python)
cpg profiler -i genome.fa -w 100 -s 10 -m gc_percent --output tsv
```

Metrics: `cg_count`, `gc_percent`, `obs_exp`.

## Output formats

All subcommands accept `--output table|tsv|json` (global flag).
