#!/usr/bin/env -S just --justfile

set windows-shell := ["powershell"]
set shell := ["bash", "-cu"]

_default:
    @just --list -u

fmt:
    @echo "Formatting all Rust code..."
    cargo fmt --all

lint:
    @echo "Running Clippy on all crates..."
    cargo clippy --workspace --all-targets --all-features -- -D warnings

fix:
    @echo "Running Clippy fix suggestions..."
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged

audit:
    @echo "Auditing Cargo.lock for vulnerabilities..."
    cargo audit

test:
    @echo "Running tests..."
    cargo test --workspace --all-features