use clap::Parser;

use cpg::cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
