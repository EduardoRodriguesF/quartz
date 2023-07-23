# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Man pages and `help` command outputs were reviewed to make it clearer..

## [0.5.0] - 2023-07-21

### Added

- UI configuration section, featuring `colors` flag to enable/disable colored outputs.
- New config command tree: `--get`, `--set` and `--list`.

### Fixed

- Piping quartz through grep or pagers would make it loose its colors.
- Releases are now automated and more reliable. Please, ignore testing versions 0.4.{0-6}...

## [0.4.6] - 2023-07-19

## [0.4.5] - 2023-07-19

## [0.4.4] - 2023-07-19

## [0.4.3] - 2023-07-19

## [0.4.2] - 2023-07-19

## [0.4.1] - 2023-07-18

## [0.4.0] - 2023-07-18

### Added

- `send` and `show` commands can receive endpoint handles for more convenient usage.

### Changed

- Terminology for endpoint pathing is now **handle**.

## [0.3.0] - 2023-07-15

### Added

- `status` command for getting current endpoint and/or context.

### Changed

- Uses `default` context in case none is specified.

### Fixed

- It is now impossible to overwrite an already existing endpoint with `create` command.
- Trying to create an endpoint without its reference would not exit with error.
- `variable` commands would always use the default context instead of the active one, causing completly broken behavior.
- Setting variables with quotation marks (e.g. `quartz variable --set baseUrl="localhost"`) would cause quartz to save the quotation marks itself. Those are now ignored.

## [0.2.1] - 2023-07-06

### Fixed
- `use` command would alter context instead of endpoint.

## [0.2.0] - 2023-07-04

### Added
- Multiple contexts can be defined to define variable values wrapped in {{<variable>}} notation when configuring an endpoint or its body. ([#14](https://github.com/EduardoRodriguesF/quartz/issues/17))
- Request history system, with the `quartz history` command to fetch it.

### Changed
- Endpoint response label "Time" has been changed to "Duration". This change was introduce to avoid confusion with Time as in Date time.

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
