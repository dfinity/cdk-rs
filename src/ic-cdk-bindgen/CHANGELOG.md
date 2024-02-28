# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

## [0.1.3] - 2024-02-27

- Resolve CANISTER_CANDID_PATH and CANISTER_ID from standardized environment variables (uppercase canister names).
  - The support for legacy (non-uppercase) env vars is kept.
  - It will be removed in next major release (v0.2).

## [0.1.2] - 2023-11-23

- Change `candid` dependency to the new `candid_parser` library.
  More details here: https://github.com/dfinity/candid/blob/master/Changelog.md#2023-11-16-rust-0100

## [0.1.1] - 2023-09-18

- Update `candid` dependency to 0.9.6 which change the Rust bindings.

## [0.1.0] - 2023-07-13

- First release.
