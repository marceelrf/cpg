use cpg::finder::criteria::{CustomCriteria, GardinerFrommer, TakaiJones};
use cpg::finder::island::find_islands;
use cpg::profiler::window::{profile, ProfileMetric};
use cpg::stats::SequenceStats;

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

#[test]
fn stats_empty_sequence() {
    let s = SequenceStats::from_seq("e", b"");
    assert_eq!(s.length, 0);
    assert_eq!(s.cg_count, 0);
    assert_eq!(s.gc_percent, 0.0);
    assert_eq!(s.obs_exp, 0.0);
}

#[test]
fn stats_known_values() {
    // CGCGCG: N=6, CpG=3, C=3, G=3 → GC=100%, Obs/Exp=2.0
    let s = SequenceStats::from_seq("t", b"CGCGCG");
    assert_eq!(s.cg_count, 3);
    assert!((s.gc_percent - 100.0).abs() < 1e-9);
    assert!((s.obs_exp - 2.0).abs() < 1e-9);
}

// ---------------------------------------------------------------------------
// Finder
// ---------------------------------------------------------------------------

#[test]
fn finder_rejects_short_sequence_gardiner() {
    // 20 bp of CG — below the 200 bp Gardiner minimum
    let seq: Vec<u8> = b"CG".iter().cycle().take(20).cloned().collect();
    assert!(find_islands("s", &seq, &GardinerFrommer).is_empty());
}

#[test]
fn finder_custom_small_window() {
    let seq    = b"CGCGCGCGCG";
    let c      = CustomCriteria { min_length: 10, min_gc: 0.5, min_obs_exp: 0.6 };
    let islands = find_islands("s", seq, &c);
    assert_eq!(islands.len(), 1);
    assert_eq!(islands[0].start,  0);
    assert_eq!(islands[0].end,   10);
}

#[test]
fn finder_two_separate_islands() {
    let mut seq: Vec<u8> = b"CG".iter().cycle().take(10).cloned().collect();
    seq.extend(b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"); // 34 A's — breaks O/E
    seq.extend(b"CG".iter().cycle().take(10).cloned());
    let c = CustomCriteria { min_length: 10, min_gc: 0.5, min_obs_exp: 0.6 };
    let islands = find_islands("s", &seq, &c);
    assert_eq!(islands.len(), 2);
}

#[test]
fn finder_takai_stricter_than_gardiner() {
    // 500 bp of pure CG: both pass, but if we lower O/E Takai should filter more
    let seq: Vec<u8> = b"CG".iter().cycle().take(500).cloned().collect();
    let g = find_islands("s", &seq, &GardinerFrommer);
    let t = find_islands("s", &seq, &TakaiJones);
    // Takai finds ≤ islands than Gardiner
    assert!(t.len() <= g.len());
}

// ---------------------------------------------------------------------------
// Profiler
// ---------------------------------------------------------------------------

#[test]
fn profiler_window_count() {
    // N=100, window=20, step=10 → 9 windows
    let seq: Vec<u8> = b"ACGT".iter().cycle().take(100).cloned().collect();
    let pts = profile(&seq, 20, 10, ProfileMetric::GcPercent);
    assert_eq!(pts.len(), 9);
}

#[test]
fn profiler_pure_cg_obs_exp_is_two() {
    let seq: Vec<u8> = b"CG".iter().cycle().take(100).cloned().collect();
    for pt in profile(&seq, 20, 10, ProfileMetric::ObsExp) {
        assert!((pt.value - 2.0).abs() < 1e-9);
    }
}

#[test]
fn profiler_gc_percent_range() {
    let seq: Vec<u8> = b"ACGT".iter().cycle().take(100).cloned().collect();
    for pt in profile(&seq, 20, 10, ProfileMetric::GcPercent) {
        assert!((0.0..=100.0).contains(&pt.value));
    }
}
