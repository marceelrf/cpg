use serde::Serialize;
use super::criteria::Criteria;
use crate::stats::metrics;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct CpgIsland {
    pub seq_id:     String,
    /// Start position, 0-based inclusive.
    pub start:      usize,
    /// End position, 0-based exclusive.
    pub end:        usize,
    pub length:     usize,
    pub cg_count:   usize,
    pub gc_percent: f64,
    pub obs_exp:    f64,
}

impl CpgIsland {
    fn from_region(seq_id: &str, seq: &[u8], start: usize, end: usize) -> Self {
        let (length, cg_count, gc_frac, obs_exp) = metrics::compute(&seq[start..end]);
        CpgIsland {
            seq_id:     seq_id.to_string(),
            start,
            end,
            length,
            cg_count,
            gc_percent: gc_frac * 100.0,
            obs_exp,
        }
    }
}

// ---------------------------------------------------------------------------
// Algorithm
// ---------------------------------------------------------------------------

/// Scan `seq` for CpG islands satisfying `criteria`.
///
/// Strategy:
/// 1. Slide a window of `min_length` bp one base at a time.
/// 2. When a window passes all three thresholds, it is a *seed*.
/// 3. Extend the seed rightward while the growing region still passes.
/// 4. Adjacent / overlapping islands are merged.
/// 5. After emitting an island, jump past its end to avoid re-scanning.
pub fn find_islands(
    seq_id:   &str,
    seq:      &[u8],
    criteria: &dyn Criteria,
) -> Vec<CpgIsland> {
    let n   = seq.len();
    let win = criteria.min_length();
    let mut islands: Vec<CpgIsland> = Vec::new();

    if n < win {
        return islands;
    }

    let mut i = 0usize;

    while i + win <= n {
        let (_, _, gc, oe) = metrics::compute(&seq[i..i + win]);

        if gc >= criteria.min_gc() && oe >= criteria.min_obs_exp() {
            // --- extend rightward ---
            let mut end = i + win;
            while end < n {
                let (_, _, gc2, oe2) = metrics::compute(&seq[i..end + 1]);
                if gc2 >= criteria.min_gc() && oe2 >= criteria.min_obs_exp() {
                    end += 1;
                } else {
                    break;
                }
            }

            // --- merge or push ---
            if let Some(last) = islands.last_mut() {
                if i < last.end {
                    // Overlaps/adjacent — extend previous island
                    let new_end = last.end.max(end);
                    *last = CpgIsland::from_region(seq_id, seq, last.start, new_end);
                    i = new_end;
                    continue;
                }
            }

            islands.push(CpgIsland::from_region(seq_id, seq, i, end));
            i = end;
        } else {
            i += 1;
        }
    }

    islands
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finder::criteria::{GardinerFrommer, CustomCriteria};

    #[test]
    fn too_short_for_gardiner() {
        // 10 bp of pure CG — below min_length=200
        let islands = find_islands("s", b"CGCGCGCGCG", &GardinerFrommer);
        assert!(islands.is_empty());
    }

    #[test]
    fn custom_small_window_detects_island() {
        let seq    = b"CGCGCGCGCG"; // 10 bp, all CG
        let custom = CustomCriteria { min_length: 10, min_gc: 0.5, min_obs_exp: 0.6 };
        let islands = find_islands("s", seq, &custom);
        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0].start, 0);
        assert_eq!(islands[0].end, 10);
    }

    #[test]
    fn gap_produces_two_islands() {
        // Two CG clusters separated by a long AT stretch
        let mut seq = b"CGCGCGCGCG".to_vec(); // island 1
        seq.extend(b"ATATATATATATATATATATATATATATATATAT"); // gap
        seq.extend(b"CGCGCGCGCG".to_vec()); // island 2
        let custom = CustomCriteria { min_length: 10, min_gc: 0.5, min_obs_exp: 0.6 };
        let islands = find_islands("s", &seq, &custom);
        assert_eq!(islands.len(), 2);
    }

    #[test]
    fn adjacent_windows_are_merged() {
        // 20 bp of pure CG — should produce one merged island, not two
        let seq    = b"CGCGCGCGCGCGCGCGCGCG";
        let custom = CustomCriteria { min_length: 10, min_gc: 0.5, min_obs_exp: 0.6 };
        let islands = find_islands("s", seq, &custom);
        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0].length, 20);
    }
}
