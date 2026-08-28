pub mod criteria;
pub mod island;

use anyhow::Result;
use bio::io::fasta;
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

    let reader = fasta::Reader::new(BufReader::new(File::open(input)?));
    let mut all_islands: Vec<CpgIsland> = Vec::new();

    for result in reader.records() {
        let record = result?;
        let mut islands = island::find_islands(
            record.id(),
            record.seq(),
            criteria.as_ref(),
        );
        all_islands.append(&mut islands);
    }

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
        OutputFormat::Table => {
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
