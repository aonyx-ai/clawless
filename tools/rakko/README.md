# Harness

This package is the harness of the Clawless repository: the one place that
states which maintenance actions run here. It mounts the bundles that this
repository uses, and the command line that it builds turns each of them into a
command. A bundle carries a set of actions that projects adopt together, so the
harness names the bundle instead of each action in it.

It also mounts commands, for a maintenance activity that no single action
describes. A command comes from a crate, as an action does, and `src/main.rs`
names it. When a command runs actions, `src/main.rs` names those actions as
well and gives them to the command.

## Usage

Run the harness from any directory of the repository:

```console
mise run rakko
```

Where mise supplies its environment, `rakko` is a shortcut for the same
command.

## Bundles

The harness mounts two bundles:

- [`rakko-baseline`][rakko-baseline] carries the actions that format and
  examine the files that any repository holds, whatever language it is
  written in, such as the formatters, the linters, and the check of the
  configuration of Renovate.
- [`rakko-rust-library`][rakko-rust-library] carries everything that a Rust
  project runs, and the checks that guard the promises a published library
  makes: the floor of each dependency, the oldest toolchain that the crates
  compile on, and the examples in the documentation.

Clawless publishes its crates as libraries, so the harness mounts the library
bundle and not `rakko-rust-binary`. Both bundles contain the same core, the
library bundle adds three checks to it, and a harness stops when two bundles
give it two actions of one name.

## Commands

### `pre-commit`

The command comes from [`rakko-pre-commit`][rakko-pre-commit]. It runs the
actions that guard a commit: the formatters first, in the order in which they
rewrite the tree, and then the checks that read what they wrote. `src/main.rs`
names both lists, and the README of the crate describes what a run does and
when it fails.

```console
mise run rakko -- pre-commit --fix
```

The hook that Git runs before a commit passes `--fix`, so that the formatters
rewrite the files that they can format. A run without the flag reports what a
commit has to repair, and it changes nothing.

The lists leave out the actions that resolve the dependencies of the project
again, such as `check-latest-deps` and `check-minimal-deps`. Each of them
reaches the network and takes minutes, which belongs in a job and not in front
of every commit. A contributor runs them by name, and CI runs them on every
pull request.

### `set-msrv`

The command comes from [`rakko-set-msrv`][rakko-set-msrv]. It sets the
minimum supported Rust version of the repository in the three places that
must agree: the root `Cargo.toml`, the Rust pin in `mise.toml`, and
`mise.lock`. The reason replaces the comment above `rust-version`.

```console
mise run rakko -- set-msrv --msrv 1.89.0 --reason "The dependency bon requires Rust 1.89.0 or later."
```

The run then installs the new toolchain through mise and runs check-msrv on
it, and it fails when the check does not pass. The files keep the new version
either way, so a failure is fixed on the same branch. `src/main.rs` gives the
command the check-msrv action.

## Layout

The package sits outside the workspace of the repository, so it resolves its
dependencies on its own and carries its own `Cargo.lock`. The binary is named
`rakko`, and the package is named `harness`, because a package that depends on
the `rakko` crates cannot carry that name as well.

Rakko builds its command line with Clawless, and it takes Clawless from
crates.io like any other dependency. The harness therefore runs on a released
version of Clawless and never on the crates in this repository. A change to
those crates cannot break the tools that check it, and the harness picks up a
new release of Clawless only when Rakko does.

Rakko publishes no release to crates.io yet, so each dependency names the
repository and the revision that this harness builds. A revision moves in one
commit, which is what a review of the update reads.

[rakko-baseline]: https://github.com/aonyx-ai/rakko/blob/main/bundles/rakko-baseline/README.md
[rakko-pre-commit]: https://github.com/aonyx-ai/rakko/blob/main/commands/rakko-pre-commit/README.md
[rakko-rust-library]: https://github.com/aonyx-ai/rakko/blob/main/bundles/rakko-rust-library/README.md
[rakko-set-msrv]: https://github.com/aonyx-ai/rakko/blob/main/commands/rakko-set-msrv/README.md
