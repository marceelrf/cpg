/// Defines the three thresholds that classify a window as a CpG island.
///
/// Implement this trait to supply your own criteria:
///
/// ```rust
/// use cpg::finder::criteria::Criteria;
///
/// struct MyCriteria;
/// impl Criteria for MyCriteria {
///     fn min_length(&self)  -> usize { 300 }
///     fn min_gc(&self)      -> f64   { 0.52 }
///     fn min_obs_exp(&self) -> f64   { 0.62 }
/// }
/// ```
pub trait Criteria: Send + Sync {
    fn min_length(&self)  -> usize;
    fn min_gc(&self)      -> f64;
    fn min_obs_exp(&self) -> f64;
}

/// Gardiner-Garden & Frommer (1987) — len≥200, GC≥50%, O/E≥0.60
#[derive(Debug, Clone, Default)]
pub struct GardinerFrommer;

impl Criteria for GardinerFrommer {
    fn min_length(&self)  -> usize { 200  }
    fn min_gc(&self)      -> f64   { 0.50 }
    fn min_obs_exp(&self) -> f64   { 0.60 }
}

/// Takai & Jones (2002) — len≥500, GC≥55%, O/E≥0.65
#[derive(Debug, Clone, Default)]
pub struct TakaiJones;

impl Criteria for TakaiJones {
    fn min_length(&self)  -> usize { 500  }
    fn min_gc(&self)      -> f64   { 0.55 }
    fn min_obs_exp(&self) -> f64   { 0.65 }
}

/// User-defined thresholds supplied via CLI flags.
#[derive(Debug, Clone)]
pub struct CustomCriteria {
    pub min_length:  usize,
    pub min_gc:      f64,
    pub min_obs_exp: f64,
}

impl Criteria for CustomCriteria {
    fn min_length(&self)  -> usize { self.min_length  }
    fn min_gc(&self)      -> f64   { self.min_gc      }
    fn min_obs_exp(&self) -> f64   { self.min_obs_exp }
}

pub fn from_preset(name: &str) -> Option<Box<dyn Criteria>> {
    match name.to_lowercase().as_str() {
        "gardiner" => Some(Box::new(GardinerFrommer)),
        "takai"    => Some(Box::new(TakaiJones)),
        _          => None,
    }
}

/// Build the effective `Criteria` from CLI arguments.
pub fn resolve(
    preset:      Option<&str>,
    min_length:  Option<usize>,
    min_gc:      Option<f64>,
    min_obs_exp: Option<f64>,
) -> anyhow::Result<Box<dyn Criteria>> {
    let base: Box<dyn Criteria> = match preset {
        Some(name) => from_preset(name).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown preset '{name}'. Valid options: gardiner, takai"
            )
        })?,
        None => Box::new(GardinerFrommer),
    };

    if min_length.is_some() || min_gc.is_some() || min_obs_exp.is_some() {
        return Ok(Box::new(CustomCriteria {
            min_length:  min_length.unwrap_or_else(|| base.min_length()),
            min_gc:      min_gc.unwrap_or_else(|| base.min_gc()),
            min_obs_exp: min_obs_exp.unwrap_or_else(|| base.min_obs_exp()),
        }));
    }

    Ok(base)
}
