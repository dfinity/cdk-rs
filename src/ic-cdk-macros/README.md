[![Documentation](https://docs.rs/ic-cdk-macros/badge.svg)](https://docs.rs/ic-cdk-macros/)
[![Crates.io](https://img.shields.io/crates/v/ic-cdk-macros.svg)](https://crates.io/crates/ic-cdk-macros)
[![License](https://img.shields.io/crates/l/ic-cdk-macros.svg)](https://github.com/dfinity/cdk-rs/blob/main/src/ic-cdk-macros/LICENSE)
[![Downloads](https://img.shields.io/crates/d/ic-cdk-macros.svg)](https://crates.io/crates/ic-cdk-macros)
[![CI](https://github.com/dfinity/cdk-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/dfinity/cdk-rs/actions/workflows/ci.yml)

# ic-cdk-macros

This crate provides a set of macros to facilitate canister development.

The macros fall into two categories:

* To register functions as canister entry points
* To export Candid definitions

## Register functions as canister entry points

These macros are directly related to the [Internet Computer Specification](https://internetcomputer.org/docs/current/references/ic-interface-spec#entry-points).

* [`init`](https://docs.rs/ic-cdk-macros/latest/ic_cdk_macros/attr.init.html)
* [`pre_upgrade`](https://docs.rs/ic-cdk-macros/latest/ic_cdk_macros/attr.pre_upgrade.html)
* [`post_upgrade`](https://docs.rs/ic-cdk-macros/latest/ic_cdk_macros/attr.post_upgrade.html)
* [`inspect_message`](https://docs.rs/ic-cdk-macros/latest/ic_cdk_macros/attr.inspect_message.html)
* [`heartbeat`](https://docs.rs/ic-cdk-macros/latest/ic_cdk_macros/attr.heartbeat.html)
* [`update`](https://docs.rs/ic-cdk-macros/latest/ic_cdk_macros/attr.update.html)
* [`query`](https://docs.rs/ic-cdk-macros/latest/ic_cdk_macros/attr.query.html)

## Export Candid definitions

* [`export_candid`](https://docs.rs/ic-cdk-macros/latest/ic_cdk_macros/macro.export_candid.html)

Check [Generating Candid files for Rust canisters](https://internetcomputer.org/docs/current/developer-docs/backend/candid/generating-candid/) for more details.
