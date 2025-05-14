# ic0

Internet Computer System API binding.

## What

`ic0` is simply an unsafe Rust translation of the System API as described in the [IC interface specification][1].

## Update and Version Strategy

`ic0` keeps in step with the IC interface specification. Particularly, `ic0` is directly generated from [system API][1] in that repo.

When interface-spec releases a new version that modify [system API][1]:

1. replace `ic0.txt` in the root of this project;
2. execute `cargo run --example=ic0build`;

`src/ic0.rs` should be updated.

The version of `ic0` crate will also bump to the same version as the IC interface specification.

[1]: https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-imports
