#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings

printf '\nPhase 0 checks: PASS\n'
