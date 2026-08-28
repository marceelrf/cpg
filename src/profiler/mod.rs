pub mod window;

use anyhow::Result;
use bio::io::fasta;
use serde::Serialize;
use std::{fs::File, io::BufReader};

use crate::cli::OutputFormat;
use window::{profile, ProfileMetric, WindowPoint};

// ---------------------------------------------------------------------------
// Output wrapper (for JSON)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct SeqProfile {
    seq_id: String,
    window: usize,
    step:   usize,
    metric: String,
    data:   Vec<WindowPoint>,
}

// ---------------------------------------------------------------------------
// Subcommand runner
// ---------------------------------------------------------------------------

pub fn run(
    input:  &str,
    window: usize,
    step:   usize,
    metric: &str,
    fmt:    &OutputFormat,
) -> Result<()> {
    let metric_enum = ProfileMetric::from_str(metric)?;
    let label       = metric_enum.label();

    let reader = fasta::Reader::new(BufReader::new(File::open(input)?));

    match fmt {
        OutputFormat::Json => {
            let mut profiles: Vec<SeqProfile> = Vec::new();
            for result in reader.records() {
                let record = result?;
                let data   = profile(record.seq(), window, step, metric_enum);
                profiles.push(SeqProfile {
                    seq_id: record.id().to_string(),
                    window,
                    step,
                    metric: label.to_string(),
                    data,
                });
            }
            println!("{}", serde_json::to_string_pretty(&profiles)?);
        }

        OutputFormat::Tsv => {
            println!("seq_id\tstart\tend\t{label}");
            for result in reader.records() {
                let record = result?;
                for pt in profile(record.seq(), window, step, metric_enum) {
                    println!("{}\t{}\t{}\t{:.4}",
                        record.id(), pt.start, pt.end, pt.value);
                }
            }
        }

        OutputFormat::Table => {
            println!("{:<25} {:>8} {:>8} {:>12}", "ID", "Start", "End", label);
            println!("{}", "─".repeat(57));
            for result in reader.records() {
                let record = result?;
                for pt in profile(record.seq(), window, step, metric_enum) {
                    println!("{:<25} {:>8} {:>8} {:>12.4}",
                        record.id(), pt.start, pt.end, pt.value);
                }
            }
        }
    }

    Ok(())
}
