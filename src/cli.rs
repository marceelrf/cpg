use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cpg", about = "CpG island analysis toolkit")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Find CpG islands using Gardiner-Garden/Frommer or Takai-Jones criteria
    Finder {
        #[arg(short, long)]
        input: String,

        #[arg(short, long, default_value = "gardiner")]
        criteria: String, // "gardiner" | "takai"
    },

    /// Compute sequence statistics (length, CG count, GC%, Obs/Exp)
    Stats {
        #[arg(short, long)]
        input: String,
    },

    /// Profile a metric over a sliding window
    Profiler {
        #[arg(short, long)]
        input: String,

        #[arg(short, long, default_value = "200")]
        window: usize,

        #[arg(short, long, default_value = "1")]
        step: usize,

        #[arg(short, long, default_value = "obs_exp")] // "cg_count" | "gc_percent" | "obs_exp"
        metric: String,
    },
}