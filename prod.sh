#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

cargo run --release -- --port 8080
