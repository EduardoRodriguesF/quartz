# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Multiple contexts can be defined to define variable values wrapped in {{<variable>}} notation when configuring an endpoint or its body.

## [0.1.6] - 2023-06-10

## [0.1.5] - 2023-06-09

### Fixed

- Infinite loop on `body --print` command ([#17](https://github.com/EduardoRodriguesF/quartz/issues/17)).
- Implement TLS for HTTPS requests to work ([#16](https://github.com/EduardoRodriguesF/quartz/issues/16)).

## [0.1.4] - 2023-06-08

### Fixed

- Edit command opening non-existant file.

## [0.1.3] - 2023-06-07

### Changed
- License switched to Apache.

## [0.1.2] - 2023-06-05

### Fixed
- Fix `send` command not working due to stack overflowing.

## [0.1.1] - 2023-06-04

### Added
- Include intallation section on documentation.

### Changed
- Change invokation to be made with `quartz`.

## [0.1.0] - 2023-06-04

### Added
- Initial release.
