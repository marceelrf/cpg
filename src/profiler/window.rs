use serde::Serialize;
use crate::stats::metrics;

#[derive(Debug, Clone, Copy)]
pub enum ProfileMetric {
    CgCount,
    GcPercent,
    ObsExp,
}

impl ProfileMetric {
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "cg_count" | "cg"   => Ok(Self::CgCount),
            "gc_percent" | "gc" => Ok(Self::GcPercent),
            "obs_exp" | "oe"    => Ok(Self::ObsExp),
            other => anyhow::bail!(
                "Unknown metric '{other}'. Valid: cg_count, gc_percent, obs_exp"
            ),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::CgCount   => "CG_count",
            Self::GcPercent => "GC%",
            Self::ObsExp    => "Obs/Exp",
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct WindowPoint {
    pub start: usize,
    pub end:   usize,
    pub value: f64,
}

/// Compute `metric` for every window of `window` bp stepping `step` bp.
pub fn profile(
    seq:    &[u8],
    window: usize,
    step:   usize,
    metric: ProfileMetric,
) -> Vec<WindowPoint> {
    let n = seq.len();
    if n < window || step == 0 { return Vec::new(); }

    let capacity = (n - window) / step + 1;
    let mut pts  = Vec::with_capacity(capacity);
    let mut start = 0usize;

    while start + window <= n {
        let end = start + window;
        let (_, cg, gc_frac, oe) = metrics::compute(&seq[start..end]);

        let value = match metric {
            ProfileMetric::CgCount   => cg as f64,
            ProfileMetric::GcPercent => gc_frac * 100.0,
            ProfileMetric::ObsExp    => oe,
        };

        pts.push(WindowPoint { start, end, value });
        start += step;
    }

    pts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_number_of_windows() {
        let seq: Vec<u8> = b"ACGT".iter().cycle().take(100).cloned().collect();
        let pts = profile(&seq, 20, 10, ProfileMetric::GcPercent);
        assert_eq!(pts.len(), 9);
    }

    #[test]
    fn seq_shorter_than_window_returns_empty() {
        let pts = profile(b"ACGT", 10, 1, ProfileMetric::ObsExp);
        assert!(pts.is_empty());
    }

    #[test]
    fn all_cg_obs_exp_is_two() {
        let seq: Vec<u8> = b"CG".iter().cycle().take(100).cloned().collect();
        for pt in profile(&seq, 20, 10, ProfileMetric::ObsExp) {
            assert!((pt.value - 2.0).abs() < 1e-9);
        }
    }
}
