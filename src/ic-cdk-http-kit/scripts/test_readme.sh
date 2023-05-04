#!/bin/bash

# Generate a temporary README file
cargo readme > readme_tmp.md

# Compare the temporary README file to the existing README.md
difference=$(diff -u --ignore-all-space README.md readme_tmp.md)

if [ -n "$difference" ]; then
  echo "[ FAIL ] README.md and generated readme_tmp.md are different:"
  echo "$difference"
  echo "Use 'cargo readme > README.md' to update README.md"
  rm readme_tmp.md
  exit 1
else
  echo "[  OK  ] README.md and generated readme_tmp.md match"
  rm readme_tmp.md
  exit 0
fi
