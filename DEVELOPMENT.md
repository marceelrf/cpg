## Custom criteria (in Rust)

Implement the `Criteria` trait to integrate `cpg` as a library:

```rust
use cpg::finder::criteria::Criteria;

struct MyCriteria;

impl Criteria for MyCriteria {
    fn min_length(&self)  -> usize { 300  }
    fn min_gc(&self)      -> f64   { 0.52 }
    fn min_obs_exp(&self) -> f64   { 0.62 }
}
```

## Running tests

```bash
cargo test
```