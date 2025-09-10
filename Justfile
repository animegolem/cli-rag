# Use bash for recipes
set shell := ["bash", "-cu"]

default: help

help:
    @just --list

# Build
dev-build:
    cargo build

build:
    cargo build --release

# Linting / formatting
fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy -- -D warnings

# Tests
test:
    cargo test

# Run binary with arbitrary args after `--`
run *args:
    cargo run --bin cli-rag -- {{args}}

# Handy shortcuts
info-json:
    just run info --format json

init-dry:
    just run init --silent --force

# Local pre-CI checks
precommit:
    scripts/line_guard.sh
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
