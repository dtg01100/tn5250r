#!/usr/bin/env bash
set -euo pipefail

# Run cargo tests with reduced parallelism and backtrace for failures
export RUST_BACKTRACE=1
export RUST_TEST_THREADS=1

# Respect .cargo/config.toml jobs, but also set a low number of parallel jobs
cargo test -- --nocapture
