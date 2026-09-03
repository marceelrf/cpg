/// Raw nucleotide/dinucleotide counts for a sequence slice.
#[derive(Debug, Default, Clone, Copy)]
pub struct RawCounts {
    pub n:        usize,
    pub c_count:  usize,
    pub g_count:  usize,
    pub cg_count: usize,
}

/// Scan `seq` once and return raw counts.
pub fn count(seq: &[u8]) -> RawCounts {
    let n = seq.len();
    let mut rc = RawCounts { n, ..Default::default() };

    for i in 0..n {
        match seq[i].to_ascii_uppercase() {
            b'C' => rc.c_count += 1,
            b'G' => rc.g_count += 1,
            _ => {}
        }
        if i + 1 < n
            && seq[i].to_ascii_uppercase()     == b'C'
            && seq[i + 1].to_ascii_uppercase() == b'G'
        {
            rc.cg_count += 1;
        }
    }
    rc
}

/// GC fraction in [0, 1].
pub fn gc_fraction(rc: &RawCounts) -> f64 {
    if rc.n == 0 { return 0.0; }
    (rc.c_count + rc.g_count) as f64 / rc.n as f64
}

/// Observed/Expected CpG ratio: (CpG × N) / (C × G).
pub fn obs_exp(rc: &RawCounts) -> f64 {
    if rc.c_count == 0 || rc.g_count == 0 { return 0.0; }
    (rc.cg_count as f64 * rc.n as f64)
        / (rc.c_count as f64 * rc.g_count as f64)
}

/// Convenience: count + derive all metrics in one call.
/// Returns `(length, cg_count, gc_fraction, obs_exp)`.
pub fn compute(seq: &[u8]) -> (usize, usize, f64, f64) {
    let rc = count(seq);
    (rc.n, rc.cg_count, gc_fraction(&rc), obs_exp(&rc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let (n, cg, gc, oe) = compute(b"");
        assert_eq!((n, cg), (0, 0));
        assert_eq!(gc, 0.0);
        assert_eq!(oe, 0.0);
    }

    #[test]
    fn pure_cg_repeat() {
        let (n, cg, gc, oe) = compute(b"CGCGCG");
        assert_eq!((n, cg), (6, 3));
        assert!((gc - 1.0).abs() < 1e-12);
        assert!((oe - 2.0).abs() < 1e-12);
    }

    #[test]
    fn gc_is_not_cpg() {
        let (_, cg, _, _) = compute(b"GCGCGC");
        assert_eq!(cg, 0);
    }

    #[test]
    fn case_insensitive() {
        let (_, cg1, gc1, _) = compute(b"cgcg");
        let (_, cg2, gc2, _) = compute(b"CGCG");
        assert_eq!(cg1, cg2);
        assert!((gc1 - gc2).abs() < 1e-12);
    }
}
