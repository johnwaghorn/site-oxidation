#!/bin/bash
set -e

echo "Running cargo fmt..."
cargo fmt --check

echo "Running cargo clippy..."
cargo clippy -- \
  -W clippy::pedantic \
  -W clippy::unwrap_used \
  -W clippy::unwrap_in_result \
  -W clippy::expect_used \
  -W clippy::panic \
  -W clippy::panic_in_result_fn \
  -W clippy::indexing_slicing \
  -W clippy::arithmetic_side_effects \
  -W clippy::as_conversions \
  -W clippy::clone_on_ref_ptr \
  -W clippy::str_to_string \
  -W clippy::implicit_clone \
  -W clippy::shadow_unrelated \
  -W clippy::missing_assert_message

echo "Running cargo test..."
cargo test
