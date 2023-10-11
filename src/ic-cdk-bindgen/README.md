# ic-cdk-bindgen

Generate Rust bindings from Candid to make inter-canister calls.

## How to use

1. Canister project add `ic-cdk-bindgen` as a build dependency.

```toml
[build-dependencies]
ic-cdk-bindgen = "0.1"
```

2. Add `build.rs` to generate Rust bindings in the source directory with config options.

```rs
use ic_cdk_bindgen::{Builder, Config};
fn main() {
    let counter = Config::new("counter");
    let mut builder = Builder::new();
    builder.add(counter);
    builder.build(None);  // default write to src/declarations
}
```

3. In Canister code,

```rs
mod declarations;
use declarations::counter::counter;

counter.inc().await?
```
