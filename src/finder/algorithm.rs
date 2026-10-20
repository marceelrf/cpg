use super::island::CpgIsland;
use super::criteria::Criteria;
use crate::stats::metrics;

// ─────────────────────────────────────────────────────────────────────────────
// Algorithm trait
// ─────────────────────────────────────────────────────────────────────────────

/// Pluggable island-detection algorithm.
pub trait Algorithm: Send + Sync {
    fn find(&self, seq_id: &str, seq: &[u8], criteria: &dyn Criteria) -> Vec<CpgIsland>;
}

// ─────────────────────────────────────────────────────────────────────────────
// SlidingWindow — the original seed-and-extend approach
// ─────────────────────────────────────────────────────────────────────────────

pub struct SlidingWindow;

impl Algorithm for SlidingWindow {
    fn find(&self, seq_id: &str, seq: &[u8], criteria: &dyn Criteria) -> Vec<CpgIsland> {
        super::island::find_islands(seq_id, seq, criteria)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RunningSum — CpGcluster-style accumulation (Hackenberg et al. 2006)
//
// Each CpG dinucleotide contributes +1 to the running sum; every non-CpG
// base decrements it by `decay`.  When the sum exceeds `min_score` a new
// island candidate starts; it ends when the sum falls back to zero.
// Candidates shorter than `criteria.min_length()` or that fail GC / O/E
// thresholds are discarded.
// ─────────────────────────────────────────────────────────────────────────────

pub struct RunningSum {
    /// Score added for every CpG dinucleotide found.
    pub cpg_score: f64,
    /// Penalty subtracted for every non-CpG base.
    pub decay: f64,
    /// Running sum must exceed this value to open a candidate island.
    pub min_score: f64,
}

impl Default for RunningSum {
    fn default() -> Self {
        RunningSum {
            cpg_score: 17.0,
            decay: 1.0,
            min_score: 0.0,
        }
    }
}

impl Algorithm for RunningSum {
    fn find(&self, seq_id: &str, seq: &[u8], criteria: &dyn Criteria) -> Vec<CpgIsland> {
        let n = seq.len();
        let mut islands: Vec<CpgIsland> = Vec::new();

        if n < 2 {
            return islands;
        }

        let mut score: f64 = 0.0;
        let mut island_start: Option<usize> = None;

        let mut i = 0usize;
        while i < n {
            let is_cpg = i + 1 < n
                && (seq[i] == b'C' || seq[i] == b'c')
                && (seq[i + 1] == b'G' || seq[i + 1] == b'g');

            if is_cpg {
                score += self.cpg_score;
                if island_start.is_none() && score > self.min_score {
                    island_start = Some(i);
                }
                i += 2; // consume both bases of the CpG
            } else {
                score -= self.decay;
                if score < 0.0 {
                    score = 0.0;
                }
                i += 1;
            }

            // Close candidate when score drops to zero
            if score == 0.0 {
                if let Some(start) = island_start.take() {
                    let end = i;
                    try_push_island(seq_id, seq, start, end, criteria, &mut islands);
                }
            }
        }

        // Close any island still open at end of sequence
        if let Some(start) = island_start.take() {
            let end = n;
            try_push_island(seq_id, seq, start, end, criteria, &mut islands);
        }

        islands
    }
}

/// Validate and push a candidate island if it meets all criteria.
fn try_push_island(
    seq_id: &str,
    seq: &[u8],
    start: usize,
    end: usize,
    criteria: &dyn Criteria,
    out: &mut Vec<CpgIsland>,
) {
    if end <= start {
        return;
    }
    let region = &seq[start..end];
    let (length, cg_count, gc_frac, obs_exp) = metrics::compute(region);
    if length >= criteria.min_length()
        && gc_frac >= criteria.min_gc()
        && obs_exp >= criteria.min_obs_exp()
    {
        // Merge with previous if overlapping
        if let Some(last) = out.last_mut() {
            if last.seq_id == seq_id && start < last.end {
                let new_end = last.end.max(end);
                let (l2, cg2, gc2, oe2) = metrics::compute(&seq[last.start..new_end]);
                *last = CpgIsland {
                    seq_id: seq_id.to_string(),
                    start: last.start,
                    end: new_end,
                    length: l2,
                    cg_count: cg2,
                    gc_percent: gc2 * 100.0,
                    obs_exp: oe2,
                };
                return;
            }
        }

        out.push(CpgIsland {
            seq_id: seq_id.to_string(),
            start,
            end,
            length,
            cg_count,
            gc_percent: gc_frac * 100.0,
            obs_exp,
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Factory
// ─────────────────────────────────────────────────────────────────────────────

/// Build an `Algorithm` from a name string.
pub fn from_name(name: &str) -> anyhow::Result<Box<dyn Algorithm>> {
    match name.to_ascii_lowercase().as_str() {
        "slidingwindow" | "sliding" | "sliding_window" => Ok(Box::new(SlidingWindow)),
        "runningsum" | "running" | "running_sum" => Ok(Box::new(RunningSum::default())),
        other => anyhow::bail!(
            "Unknown algorithm '{other}'. Valid options: slidingwindow | runningsum"
        ),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finder::criteria::CustomCriteria;

    fn small_criteria() -> CustomCriteria {
        CustomCriteria {
            min_length: 10,
            min_gc: 0.5,
            min_obs_exp: 0.6,
        }
    }

    #[test]
    fn sliding_window_basic() {
        let seq = b"CGCGCGCGCG";
        let islands = SlidingWindow.find("s", seq, &small_criteria());
        assert_eq!(islands.len(), 1);
    }

    #[test]
    fn running_sum_finds_island() {
        // Use a very lenient criteria so the RunningSum island passes validation
        let lenient = CustomCriteria { min_length: 10, min_gc: 0.3, min_obs_exp: 0.3 };
        // Dense CpG stretch (40 CpGs = 80 bp) flanked by AT-rich sequence
        let mut seq = b"ATATATATATATATATATATAT".to_vec();
        for _ in 0..40 { seq.extend(b"CG"); }
        seq.extend(b"ATATATATATATATATATATAT");
        let islands = RunningSum::default().find("s", &seq, &lenient);
        assert!(!islands.is_empty(), "expected at least one island");
    }

    #[test]
    fn from_name_slidingwindow() {
        assert!(from_name("slidingwindow").is_ok());
        assert!(from_name("SlidingWindow").is_ok());
    }

    #[test]
    fn from_name_runningsum() {
        assert!(from_name("runningsum").is_ok());
        assert!(from_name("running_sum").is_ok());
    }

    #[test]
    fn from_name_invalid() {
        assert!(from_name("nonsense").is_err());
    }
}
