#!/bin/bash
set -e

echo "Running cargo fmt..."
cargo fmt --check

echo "Running cargo clippy..."
cargo clippy --all -- -W clippy::all -W clippy::pedantic

echo "Running cargo test..."
cargo test
