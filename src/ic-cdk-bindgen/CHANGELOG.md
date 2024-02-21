# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

- Look up the `CANISTER_ID_<canister_name_uppercase>` and `CANISTER_CANDID_PATH_<canister_name_uppercase>` 
  environment variables to get the canister ID for a canister name. Previously the case of the canister
  name matched the case of the canister, typically lowercase.

## [0.1.2] - 2023-11-23

- Change `candid` dependency to the new `candid_parser` library.
  More details here: https://github.com/dfinity/candid/blob/master/Changelog.md#2023-11-16-rust-0100

## [0.1.1] - 2023-09-18

- Update `candid` dependency to 0.9.6 which change the Rust bindings.

## [0.1.0] - 2023-07-13

- First release.
