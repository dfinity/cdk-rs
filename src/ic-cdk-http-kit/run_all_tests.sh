#!/bin/sh

set -e

# Check if README.md is up-to-date.
echo "Checking if README.md is up-to-date..."
./scripts/test_readme.sh
echo "README.md is up-to-date."

# Run cargo tests for the crate.
echo "Running cargo tests for the crate..."
cargo test
echo "Cargo tests for the crate passed."

# Run cargo tests for example projects.
echo "Running cargo tests for example projects..."
(
  cd examples
  cargo test
)
echo "Cargo tests for example projects passed."

# Run dfx end-to-end tests for specific example projects.
echo "Running dfx end-to-end tests for specific example projects..."
(
  cd examples/fetch_json
  e2e-tests/fetch_quote.sh
)
echo "dfx end-to-end tests for specific example projects passed."

# All tests passed
echo "All tests passed successfully."
