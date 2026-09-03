use serde::Serialize;
use super::criteria::Criteria;
use crate::stats::metrics;

#[derive(Debug, Serialize, Clone)]
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
            seq_id: seq_id.to_string(),
            start,
            end,
            length,
            cg_count,
            gc_percent: gc_frac * 100.0,
            obs_exp,
        }
    }
}

/// Scan `seq` for CpG islands satisfying `criteria`.
pub fn find_islands(
    seq_id:   &str,
    seq:      &[u8],
    criteria: &dyn Criteria,
) -> Vec<CpgIsland> {
    let n   = seq.len();
    let win = criteria.min_length();
    let mut islands: Vec<CpgIsland> = Vec::new();

    if n < win { return islands; }

    let mut i = 0usize;

    while i + win <= n {
        let (_, _, gc, oe) = metrics::compute(&seq[i..i + win]);

        if gc >= criteria.min_gc() && oe >= criteria.min_obs_exp() {
            // Extend rightward
            let mut end = i + win;
            while end < n {
                let (_, _, gc2, oe2) = metrics::compute(&seq[i..end + 1]);
                if gc2 >= criteria.min_gc() && oe2 >= criteria.min_obs_exp() {
                    end += 1;
                } else {
                    break;
                }
            }

            // Merge with previous if overlapping
            if let Some(last) = islands.last_mut() {
                if i < last.end {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finder::criteria::{GardinerFrommer, CustomCriteria};

    #[test]
    fn too_short_for_gardiner() {
        let islands = find_islands("s", b"CGCGCGCGCG", &GardinerFrommer);
        assert!(islands.is_empty());
    }

    #[test]
    fn custom_small_window_detects_island() {
        let seq    = b"CGCGCGCGCG";
        let custom = CustomCriteria { min_length: 10, min_gc: 0.5, min_obs_exp: 0.6 };
        let islands = find_islands("s", seq, &custom);
        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0].start, 0);
        assert_eq!(islands[0].end, 10);
    }

    #[test]
    fn gap_produces_two_islands() {
        let mut seq = b"CGCGCGCGCG".to_vec();
        seq.extend(b"ATATATATATATATATATATATATATATATATAT");
        seq.extend(b"CGCGCGCGCG".to_vec());
        let custom = CustomCriteria { min_length: 10, min_gc: 0.5, min_obs_exp: 0.6 };
        let islands = find_islands("s", &seq, &custom);
        assert_eq!(islands.len(), 2);
    }

    #[test]
    fn adjacent_windows_are_merged() {
        let seq    = b"CGCGCGCGCGCGCGCGCGCG";
        let custom = CustomCriteria { min_length: 10, min_gc: 0.5, min_obs_exp: 0.6 };
        let islands = find_islands("s", seq, &custom);
        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0].length, 20);
    }
}
