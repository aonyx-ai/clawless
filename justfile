# Run all recipes inside the Flox environment
set shell := ["flox", "activate", "--", "sh", "-cu"]

# Commands to build and serve the documentation site
mod docs

[private]
default:
    @just --list

# Run a subset of checks as pre-commit hooks
pre-commit: pre-commit-fix pre-commit-verify

# Every recipe that rewrites the working tree, in sequence: they overlap each
# other, and nothing may read a file while one of them is writing it.
[private]
pre-commit-fix:
    just prettier true
    just format-toml true
    just format-rust true

# Every recipe that only reads, in parallel: the tree has stopped changing, so
# what each of them sees is what the commit will contain.
[private]
pre-commit-verify:
    #!/usr/bin/env -S flox activate -- parallel --shebang --ungroup --jobs {{ num_cpus() }}
    just lint-github-actions
    just lint-markdown
    just lint-rust
    just lint-yaml
    just test-rust

# Build the documentation for the crates
build-rust-docs:
    cargo doc --all-features --no-deps

# Build the public website
build-website:
    just docs build

# Check that clawless builds with the latest dependencies
check-latest-deps force="false":
    #!/usr/bin/env bash

    # Abort if git is not clean (but ignore Flox's manifest.lock)
    if [[ {{force}} != "true" && -n $(git status --porcelain -- ':!.flox/env/manifest.lock') ]]; then
        echo "Git working directory is not clean. Commit or stash changes before running this recipe. Aborting."
        git status --porcelain

        # Print diff on GitHub Actions
        if [ -n "$GITHUB_ACTIONS" ]; then
            git diff
        fi

        exit 1
    fi

    # Update dependencies to latest versions
    cargo update

    # Run tests to ensure the latest versions are compatible
    RUSTFLAGS="-D deprecated" cargo test --all-features --all-targets --locked

# Check that dependencies have compatible open-source licenses and trusted sources
check-dependencies:
    cargo deny check bans licenses sources

# Check that clawless builds with the minimal dependencies
check-minimal-deps force="false":
    #!/usr/bin/env bash

    # Abort if git is not clean (but ignore Flox's manifest.lock)
    if [[ {{force}} != "true" && -n $(git status --porcelain -- ':!.flox/env/manifest.lock') ]]; then
        echo "Git working directory is not clean. Commit or stash changes before running this recipe. Aborting."
        git status --porcelain

        # Print diff on GitHub Actions
        if [ -n "$GITHUB_ACTIONS" ]; then
            git diff
        fi

        exit 1
    fi

    # Install the nightly toolchain if not already installed
    rustup install nightly

    # Update dependencies to minimal versions
    rustup run nightly cargo update -Z direct-minimal-versions

    # Run tests to ensure the minimal versions are compatible
    RUSTFLAGS="-D deprecated" rustup run nightly cargo test --all-features --all-targets --locked

# Check that clawless builds with the MSRV
check-msrv:
    #!/usr/bin/env bash

    # Get the MSRV from the Cargo.toml
    MSRV=$(cat Cargo.toml | grep 'rust-version =' | head -n 1 | cut -d '"' -f 2)

    # Install the MSRV toolchain if not already installed
    rustup install "${MSRV}"

    # Run tests using the MSRV
    RUSTFLAGS="-D deprecated" rustup run "${MSRV}" cargo check --all-features --all-targets

# Check that all dependencies in Cargo.toml are used
check-unused-deps:
    #!/usr/bin/env bash

    # Install the nightly toolchain if not already installed
    rustup install nightly

    # Check for unused dependencies
    rustup run nightly cargo udeps

# Format JSON files
format-json fix="false": (prettier fix "{json,json5}")

# Format Markdown files
format-markdown fix="false": (prettier fix "md")

# Format Rust files
format-rust fix="false":
    rustup install -c rustfmt nightly
    rustup run nightly cargo fmt -- --unstable-features {{ if fix != "true" { "--check" } else { "" } }}

# Format TOML files
format-toml fix="false":
    taplo fmt {{ if fix != "true" { "--diff" } else { "" } }}

# Format YAML files
format-yaml fix="false": (prettier fix "{yaml,yml}")

# Lint GitHub Actions workflows
lint-github-actions:
    zizmor -p .

# Lint Markdown files
#
# The glob is quoted so that markdownlint expands it. The recipe shell is
# `sh`, which has no globstar, so an unquoted `**/*.md` collapses to
# `*/*.md` and reaches only the files one directory deep.
lint-markdown:
    markdownlint "**/*.md"

# Lint Rust files
lint-rust:
    cargo clippy --all-targets --all-features -- -D warnings

# Lint TOML files
lint-toml:
    taplo check

# Lint YAML files
lint-yaml:
    yamllint .

# Auto-format files with prettier
[private]
prettier fix="false" extension="*":
    prettier {{ if fix == "true" { "--write" } else { "--list-different" } }} --ignore-unknown "**/*.{{ extension }}"

# Publish the crates to crates.io
publish:
    cargo publish --all-features --verbose --workspace

# Run the tests
test-rust:
    cargo nextest run --all-features --all-targets
