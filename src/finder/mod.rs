pub mod criteria;
pub mod island;

use anyhow::Result;
use bio::io::fasta;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::{fs::File, io::BufReader};

use crate::cli::OutputFormat;
use island::CpgIsland;

pub fn run(
    input:       &str,
    preset:      Option<&str>,
    min_length:  Option<usize>,
    min_gc:      Option<f64>,
    min_obs_exp: Option<f64>,
    fmt:         &OutputFormat,
) -> Result<()> {
    let criteria = criteria::resolve(preset, min_length, min_gc, min_obs_exp)?;

    // --- load all records first (needed for parallel processing) ---
    let reader = fasta::Reader::new(BufReader::new(File::open(input)?));
    let records: Vec<(String, Vec<u8>)> = reader
        .records()
        .map(|r| {
            let r = r?;
            Ok((r.id().to_string(), r.seq().to_vec()))
        })
        .collect::<Result<_>>()?;

    // --- progress spinner (goes to stderr, doesn't interfere with output) ---
    let spinner = ProgressBar::new(records.len() as u64);
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} [{pos}/{len}] {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]),
    );

    // --- parallel island detection ---
    let all_islands: Vec<CpgIsland> = records
        .par_iter()
        .flat_map(|(id, seq)| {
            spinner.set_message(format!("Processing {id}"));
            let islands = island::find_islands(id, seq, criteria.as_ref());
            spinner.inc(1);
            islands
        })
        .collect();

    spinner.finish_with_message(format!(
        "Done — {} island(s) found",
        all_islands.len()
    ));

    // --- output ---
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&all_islands)?);
        }

        OutputFormat::Tsv => {
            println!("seq_id\tstart\tend\tlength\tcg_count\tgc_percent\tobs_exp");
            for isl in &all_islands {
                println!("{}\t{}\t{}\t{}\t{}\t{:.4}\t{:.4}",
                    isl.seq_id, isl.start, isl.end, isl.length,
                    isl.cg_count, isl.gc_percent, isl.obs_exp);
            }
        }

        // BED: chrom  chromStart  chromEnd  name  score  strand
        OutputFormat::Bed => {
            for (i, isl) in all_islands.iter().enumerate() {
                println!("{}\t{}\t{}\tCpGisland_{}\t0\t.",
                    isl.seq_id, isl.start, isl.end, i + 1);
            }
        }

        OutputFormat::Table | OutputFormat::Bedgraph => {
            println!("{:<25} {:>8} {:>8} {:>8} {:>8} {:>9} {:>10}",
                "ID", "Start", "End", "Length", "CG", "GC%", "Obs/Exp");
            println!("{}", "─".repeat(82));
            for isl in &all_islands {
                println!("{:<25} {:>8} {:>8} {:>8} {:>8} {:>8.2}% {:>10.4}",
                    isl.seq_id, isl.start, isl.end, isl.length,
                    isl.cg_count, isl.gc_percent, isl.obs_exp);
            }
            if all_islands.is_empty() {
                println!("  (no CpG islands found)");
            }
        }
    }

    Ok(())
}
