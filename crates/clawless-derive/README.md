# 🦦 `clawless-derive`

`clawless` is a framework for building command-line applications with Rust, and
the `clawless-derive` crate implements the procedural macros that power this
framework.

The crate defines the `app!` macro and the `#[command]` macro attribute, which
the `clawless` crate re-exports. The `app!` macro generates a noop `#[command]`
as the root of the command-line application, while the `#[command]` macro
attribute does the heavy lifting of creating a command and registering it with
its parent.

Check the documentation of the two macros to get a better understanding of how
this crate works.

## License

Licensed under either of

- Apache License, Version 2.0 (<http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
