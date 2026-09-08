use clap::Parser;
use rayon;
use cpg::cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(n) = cli.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .map_err(|e| anyhow::anyhow!("Failed to build thread pool: {e}"))?;
    }

    match cli.command {
        Commands::Finder {
            input,
            criteria,
            min_length,
            min_gc,
            min_obs_exp,
        } => cpg::finder::run(
            &input,
            criteria.as_deref(),
            min_length,
            min_gc,
            min_obs_exp,
            &cli.output,
        ),

        Commands::Stats { input } => {
            cpg::stats::run(&input, &cli.output)
        }

        Commands::Profiler { input, window, step, metric } => {
            cpg::profiler::run(&input, window, step, &metric, &cli.output)
        }
    }
}
