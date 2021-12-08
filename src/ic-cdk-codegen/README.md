# ic-cdk-codegen

Codegen backend for [`ic_cdk_macros`](https://docs.rs/ic-cdk-macros). The
intended use of this crate is indirectly via [`#[ic_cdk_macros::import]`][import]
but you can also use this in a build script to pregenerate the code.

[import]: https://docs.rs/ic-cdk-macros/*/ic_cdk_macros/attr.import.html
