pub mod metrics;

use anyhow::Result;
use bio::io::fasta;
use serde::Serialize;
use std::{fs::File, io::BufReader};

use crate::cli::OutputFormat;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SequenceStats {
    pub id:         String,
    pub length:     usize,
    pub cg_count:   usize,
    pub gc_percent: f64,
    pub obs_exp:    f64,
}

impl SequenceStats {
    pub fn from_seq(id: impl Into<String>, seq: &[u8]) -> Self {
        let (length, cg_count, gc_frac, obs_exp) = metrics::compute(seq);
        Self {
            id: id.into(),
            length,
            cg_count,
            gc_percent: gc_frac * 100.0,
            obs_exp,
        }
    }
}

// ---------------------------------------------------------------------------
// Subcommand runner
// ---------------------------------------------------------------------------

pub fn run(input: &str, fmt: &OutputFormat) -> Result<()> {
    let reader = fasta::Reader::new(BufReader::new(File::open(input)?));
    let records: Vec<SequenceStats> = reader
        .records()
        .map(|r| {
            let r = r?;
            Ok(SequenceStats::from_seq(r.id(), r.seq()))
        })
        .collect::<Result<_>>()?;

    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&records)?);
        }
        OutputFormat::Tsv => {
            println!("id\tlength\tcg_count\tgc_percent\tobs_exp");
            for s in &records {
                println!("{}\t{}\t{}\t{:.4}\t{:.4}",
                    s.id, s.length, s.cg_count, s.gc_percent, s.obs_exp);
            }
        }
        OutputFormat::Table => {
            println!("{:<25} {:>10} {:>10} {:>10} {:>10}",
                "ID", "Length", "CG_count", "GC%", "Obs/Exp");
            println!("{}", "─".repeat(69));
            for s in &records {
                println!("{:<25} {:>10} {:>10} {:>9.2}% {:>10.4}",
                    s.id, s.length, s.cg_count, s.gc_percent, s.obs_exp);
            }
        }
    }

    Ok(())
}
