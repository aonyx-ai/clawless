<!-- markdownlint-disable-file MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-12-19

### Added

- Pass the current working directory to commands
- Add a command to `clawless-cli` to scaffold new Clawless applications
- Add a command to `clawless-cli` to generate new commands
- Add support for command aliases

### Changed

- Pass a `Context` to commands
- Rename the `noop` attribute to `require_subcommand`

## [0.3.0] - 2025-11-21

### Added

- Document CLIs using doc comments
- Return result type from commands
- Return `Result` from `clawless::main!`

### Changed

- Refactor the macros to simplify the crate's API
- Rename the error types
- Upgrade to Rust edition 2024
- Re-export clap dependency
- Reintroduce `commands` module

## [0.2.0] - 2025-07-11

### Changed

- Increase minimum version of `proc-macro2` dependency to `1.0.86`

## [0.1.0] - 2025-06-20

### Added

- Initial prototype featuring the `clawless!`, `app!`, and `#[command]` macros

[0.4.0]: https://github.com/aonyx-ai/clawless/releases/tag/v0.4.0
[0.3.0]: https://github.com/aonyx-ai/clawless/releases/tag/v0.3.0
[0.2.0]: https://github.com/aonyx-ai/clawless/releases/tag/v0.2.0
[0.1.0]: https://github.com/aonyx-ai/clawless/releases/tag/v0.1.0
