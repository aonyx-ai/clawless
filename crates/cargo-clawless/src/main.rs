#![cfg_attr(not(doctest),doc = include_str!("../README.md"))]
#![allow(
    unreachable_pub,
    reason = "this crate compiles to a binary, so nothing in it is reachable from outside the crate and `unreachable_pub` would demand `pub(crate)` on every item. The lint earns its keep in the library crates, where the public API is a real boundary"
)]

mod commands;
mod input;

clawless::main!();
