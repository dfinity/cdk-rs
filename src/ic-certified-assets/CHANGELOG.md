# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.5] - 2022-08-22
### Removed 
- Support for asset caching based on [ETag](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag)
- Automatic redirection of all traffic from `.raw.ic0.app` domain to `.ic0.app`

## [0.2.4] - 2022-07-12
### Fixed
- headers field in Candid spec accepts mmultiple HTTP headers

## [0.2.3] - 2022-07-06
### Added
- Support for setting custom HTTP headers on asset creation 

## [0.2.2] - 2022-05-12
### Fixed
- Parse and produce ETag headers with quotes around the hash

## [0.2.1] - 2022-05-12
### Fixed
- Make StableState public again

## [0.2.0] - 2022-05-11
### Added
- Support for asset caching based on [ETag](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag)
- Support for asset caching based on [max-age](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control)
- Automatic redirection of all traffic from `.raw.ic0.app` domain to `.ic0.app`

## [0.1.0] - 2022-02-02
### Added
- First release
