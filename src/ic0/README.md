# ic0

Internet Computer System API binding.

## What

`ic0` is simply an unsafe Rust translation of Internet Computer System API as described in the [Specification](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-imports).

## Update and Version Strategy

`ic0` keeps in step with [interface-spec][1]. Particularly, `ic0` is directly generated from [ic0.txt][2] in that repo.

When interface-spec releases a new version that modify [ic0.txt][2], we replace `ic0.txt` in the root of this crate and run `cargo build` to generate a new `src/ic0.rs`.

The version of `ic0` crate will also bump to the same version as [interface-spec][1].

[1]: https://github.com/dfinity/interface-spec
[2]: https://github.com/dfinity/interface-spec/blob/master/spec/ic0.txt
