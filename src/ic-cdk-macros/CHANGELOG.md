# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

## [0.8.1] - 2023-10-02

### Fixed

- Macros no longer use global state in the names of functions. JetBrains IDEs should no longer produce spurious errors. (#430)

## [0.8.0] - 2023-09-18

### Changed

- Remove `export_candid` feature. (#424)

### Fixed

- Export composite_query to Candid. (#419)

## [0.7.1] - 2023-07-27

### Fixed

- Only update/query macros can take guard function. (#417)

## [0.7.0] - 2023-07-13

### Added

- `export_candid` macro. (#386)

### Changed

- Remove `import` macro. (#390)

## [0.6.10] - 2023-03-01

### Changed

- Update lint settings.
