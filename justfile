# Run all recipes inside the Flox environment
set shell := ["flox", "activate", "--", "sh", "-cu"]

[private]
default:
    @just --list

# Run a subset of checks as pre-commit hooks
pre-commit:
    #!/usr/bin/env -S parallel --shebang --ungroup --jobs {{ num_cpus() }}
    just prettier true
    just format-toml true
    just format-rust true
    just lint-github-actions
    just lint-markdown
    just lint-rust
    just lint-yaml
    just test-rust

# Build the documentation for the crates
build-docs:
    cargo doc --all-features --no-deps

# Check that clawless builds with the latest dependencies
check-latest-deps:
    #!/usr/bin/env bash

    # Abort if git is not clean (but ignore Flox's manifest.lock)
    if [[ -n $(git status --porcelain -- ':!.flox/env/manifest.lock') ]]; then
        echo "Git working directory is not clean. Commit or stash changes before running this recipe. Aborting."
        git status --porcelain
        exit 1
    fi

    # Update dependencies to latest versions
    cargo update

    # Run tests to ensure the latest versions are compatible
    RUSTFLAGS="-D deprecated" cargo test --all-features --all-targets --locked

# Check that clawless builds with the minimal dependencies
check-minimal-deps:
    #!/usr/bin/env bash

    # Abort if git is not clean (but ignore Flox's manifest.lock)
    if [[ -n $(git status --porcelain -- ':!.flox/env/manifest.lock') ]]; then
        echo "Git working directory is not clean. Commit or stash changes before running this recipe. Aborting."
        git status --porcelain
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
    cargo fmt {{ if fix != "true" { "--check" } else { "" } }}

# Format TOML files
format-toml fix="false":
    taplo fmt {{ if fix != "true" { "--diff" } else { "" } }}

# Format YAML files
format-yaml fix="false": (prettier fix "{yaml,yml}")

# Lint GitHub Actions workflows
lint-github-actions:
    zizmor -p .

# Lint Markdown files
lint-markdown:
    markdownlint **/*.md

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
    cargo publish -p clawless-derive -v --all-features
    cargo publish -p clawless -v --all-features

# Run the tests
test-rust:
    cargo test --all-features --all-targets
