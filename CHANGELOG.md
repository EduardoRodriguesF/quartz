# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Added

- `send` command new edit flags: `-d,--data <DATA>`, `-H, --header <HEADER>`, `--query <QUERY>`, `--var <VARIABLE>`, `-X, --request <METHOD>`.
- `use` command can now edit the current or to-be-used endpoint.

## Changed

- `header`, `query`, `config` and `variable` commands now follow the same new pattern to promote consistency.
- `create` options were revisited for better semantics with **curl** and other `send` and `use` options.

## [0.8.0] - 2023-08-12

## Added

- New `last` command tree.
- `--show <FIELDS>` to `history` and `send` commands to specify fields to be shown on listing.

## Changed

- History saving format changed, which probably breaks previously saved entries.

## [0.7.1] - 2023-08-05

## Changed

- `variables` options `--set` and `--edit` are now executed in that order. Changes from `--set` will be committed before `--edit` comes in.

## Fixed

- It was possible to save malformed files through `edit` and `var --edit` commands. From now on, a parsing error is thrown. [#30](https://github.com/EduardoRodriguesF/quartz/issues/30)
- Linux with aarch64 architecture were incompatible with quartz.

## [0.7.0] - 2023-07-29

## Added

- More convenient `--get` option for `header` command.
- It is now possible to use multiple `--set` in a single `variable` command.
- New `--apply-context` option to apply context variables as soon as possible.

## Changed

- `headers` command was renamed to `header`.
- Headers option `--add` was renamed to `--set`.

## Fixed

- Query params could not use context variables.
- Some outputs were inconsistent.

## [0.6.0] - 2023-07-25

## Added

- New create `--query` option to use new query params system.
- New url `--full` option to print with queries.
- Query params can be defined separate from URL through the `query` command.
- New `-x` option for temporary handle switch.

## Changed

- [**BREAKING CHANGE**] Handles are now separated by slash (/). What used to be "auth users create" is now "auth/users/create".
- List command outputs a flat list of all handles to improve readability.

## [0.5.1] - 2023-07-23

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
