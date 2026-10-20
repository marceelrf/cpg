/// A genomic region filter supplied via `--region chr:start-end`.
#[derive(Debug, Clone)]
pub struct Region {
    pub chrom: String,
    pub start: usize, // 0-based inclusive
    pub end: usize,   // 0-based exclusive
}

impl Region {
    /// Parse a string like `chr1:1000-5000` or `NM_000518:0-2000`.
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let (chrom, coords) = s.split_once(':').ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid region '{s}'. Expected format: chrom:start-end  (e.g. chr1:1000-5000)"
            )
        })?;

        let (start_s, end_s) = coords.split_once('-').ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid region '{s}'. Expected format: chrom:start-end  (e.g. chr1:1000-5000)"
            )
        })?;

        let start: usize = start_s.trim().parse().map_err(|_| {
            anyhow::anyhow!("Region start '{}' is not a valid integer", start_s.trim())
        })?;

        let end: usize = end_s.trim().parse().map_err(|_| {
            anyhow::anyhow!("Region end '{}' is not a valid integer", end_s.trim())
        })?;

        if start >= end {
            anyhow::bail!("Region start ({start}) must be less than end ({end})");
        }

        Ok(Region {
            chrom: chrom.trim().to_string(),
            start,
            end,
        })
    }

    /// Returns `true` if `seq_id` matches and the interval `[isl_start, isl_end)`
    /// overlaps with this region.
    pub fn contains(&self, seq_id: &str, isl_start: usize, isl_end: usize) -> bool {
        seq_id == self.chrom && isl_start < self.end && isl_end > self.start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid() {
        let r = Region::parse("chr1:1000-5000").unwrap();
        assert_eq!(r.chrom, "chr1");
        assert_eq!(r.start, 1000);
        assert_eq!(r.end, 5000);
    }

    #[test]
    fn parse_missing_colon() {
        assert!(Region::parse("chr11000-5000").is_err());
    }

    #[test]
    fn parse_start_gte_end() {
        assert!(Region::parse("chr1:5000-1000").is_err());
    }

    #[test]
    fn contains_overlap() {
        let r = Region::parse("chr1:1000-2000").unwrap();
        assert!(r.contains("chr1", 500, 1500)); // overlaps left
        assert!(r.contains("chr1", 1200, 1800)); // fully inside
        assert!(r.contains("chr1", 1800, 2500)); // overlaps right
        assert!(!r.contains("chr1", 0, 999)); // before
        assert!(!r.contains("chr1", 2001, 3000)); // after
        assert!(!r.contains("chr2", 1200, 1800)); // wrong chrom
    }
}
