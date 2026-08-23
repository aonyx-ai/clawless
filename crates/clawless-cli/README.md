# 🦦 clawless-cli

`clawless` is an opinionated, batteries-included framework for building
command-line applications with Rust. It features low-level building blocks and
high-level abstractions that can be used to build stateless CLIs or stateful
TUI applications.

This crate, `clawless-cli`, implements the CLI presentation layer of the
framework. It contains the command execution context, the output
configuration, and the push-based presenter that renders command output to the
terminal as events arrive.

Use the [`clawless`] facade crate instead of this crate. The facade re-exports
everything from this crate alongside [`clawless-core`] and
[`clawless-derive`], so that CLI applications need only a single dependency.

## History

Before version 0.6.0, `clawless-cli` was the scaffolding tool for the
framework, and `cargo install clawless-cli` installed the `clawless` binary.
The scaffolding tool is now in [`cargo-clawless`], and this crate no longer
contains a binary. To scaffold projects and generate commands, install the new
crate instead:

```shell
cargo install cargo-clawless
```

## License

Licensed under either of

- Apache License, Version 2.0 (<http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[`cargo-clawless`]: https://crates.io/crates/cargo-clawless
[`clawless`]: https://docs.rs/clawless
[`clawless-core`]: https://docs.rs/clawless-core
[`clawless-derive`]: https://docs.rs/clawless-derive
