use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name        = "cpg",
    version,
    about       = "CpG island analysis toolkit",
    long_about  = "Detect CpG islands, compute sequence statistics, and profile \
                   metrics in sliding windows — all from FASTA input."
)]
pub struct Cli {
    /// Output format
    #[arg(long, global = true, default_value = "table", value_enum)]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Tsv,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Find CpG islands using built-in or fully custom criteria
    ///
    /// Built-in presets (--criteria):
    ///   gardiner  — Gardiner-Garden & Frommer 1987 (len≥200, GC≥50%, O/E≥0.60)
    ///   takai     — Takai & Jones 2002            (len≥500, GC≥55%, O/E≥0.65)
    ///
    /// Any flag (--min-length, --min-gc, --min-obs-exp) overrides the preset value.
    /// Omitting --criteria AND all override flags defaults to "gardiner".
    Finder {
        /// Input FASTA file
        #[arg(short, long)]
        input: String,

        /// Built-in criteria preset: gardiner | takai
        #[arg(short, long, value_name = "PRESET")]
        criteria: Option<String>,

        /// Override minimum island length (bp)
        #[arg(long, value_name = "BP")]
        min_length: Option<usize>,

        /// Override minimum GC content [0.0 – 1.0]
        #[arg(long, value_name = "FRACTION")]
        min_gc: Option<f64>,

        /// Override minimum observed/expected CpG ratio
        #[arg(long, value_name = "RATIO")]
        min_obs_exp: Option<f64>,
    },

    /// Compute per-sequence statistics: length, CG count, GC%, Obs/Exp
    Stats {
        /// Input FASTA file
        #[arg(short, long)]
        input: String,
    },

    /// Profile a metric along the sequence using a sliding window
    Profiler {
        /// Input FASTA file
        #[arg(short, long)]
        input: String,

        /// Window size (bp)
        #[arg(short, long, default_value_t = 200, value_name = "BP")]
        window: usize,

        /// Step size between windows (bp)
        #[arg(short, long, default_value_t = 50, value_name = "BP")]
        step: usize,

        /// Metric to profile: cg_count | gc_percent | obs_exp
        #[arg(short, long, default_value = "obs_exp", value_name = "METRIC")]
        metric: String,
    },
}
