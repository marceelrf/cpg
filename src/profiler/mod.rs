pub mod window;

use anyhow::Result;
use bio::io::fasta;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Serialize;
use std::{fs::File, io::BufReader};

use crate::cli::OutputFormat;
use window::{profile, ProfileMetric, WindowPoint};

#[derive(Serialize)]
struct SeqProfile {
    seq_id: String,
    window: usize,
    step:   usize,
    metric: String,
    data:   Vec<WindowPoint>,
}

pub fn run(
    input:  &str,
    window: usize,
    step:   usize,
    metric: &str,
    fmt:    &OutputFormat,
) -> Result<()> {
    let metric_enum = ProfileMetric::from_str(metric)?;
    let label       = metric_enum.label();

    // --- load all records ---
    let reader = fasta::Reader::new(BufReader::new(File::open(input)?));
    let records: Vec<(String, Vec<u8>)> = reader
        .records()
        .map(|r| {
            let r = r?;
            Ok((r.id().to_string(), r.seq().to_vec()))
        })
        .collect::<Result<_>>()?;

    // --- progress spinner ---
    let spinner = ProgressBar::new(records.len() as u64);
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} [{pos}/{len}] {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]),
    );

    // --- parallel profiling ---
    let profiles: Vec<SeqProfile> = records
        .par_iter()
        .map(|(id, seq)| {
            spinner.set_message(format!("Profiling {id}"));
            let data = profile(seq, window, step, metric_enum);
            spinner.inc(1);
            SeqProfile {
                seq_id: id.clone(),
                window,
                step,
                metric: label.to_string(),
                data,
            }
        })
        .collect();

    spinner.finish_with_message(format!(
        "Done — {} sequence(s) profiled",
        profiles.len()
    ));

    // --- output ---
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&profiles)?);
        }

        OutputFormat::Tsv => {
            println!("seq_id\tstart\tend\t{label}");
            for p in &profiles {
                for pt in &p.data {
                    println!("{}\t{}\t{}\t{:.4}", p.seq_id, pt.start, pt.end, pt.value);
                }
            }
        }

        // BEDGRAPH: chrom  chromStart  chromEnd  dataValue
        OutputFormat::Bedgraph => {
            for p in &profiles {
                for pt in &p.data {
                    println!("{}\t{}\t{}\t{:.6}", p.seq_id, pt.start, pt.end, pt.value);
                }
            }
        }

        OutputFormat::Table | OutputFormat::Bed => {
            println!("{:<25} {:>8} {:>8} {:>12}", "ID", "Start", "End", label);
            println!("{}", "─".repeat(57));
            for p in &profiles {
                for pt in &p.data {
                    println!("{:<25} {:>8} {:>8} {:>12.4}",
                        p.seq_id, pt.start, pt.end, pt.value);
                }
            }
        }
    }

    Ok(())
}
