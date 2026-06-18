#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo fetch --locked
cargo fmt --check
cargo test --locked
