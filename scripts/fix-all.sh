#!/usr/bin/env bash

set -euo pipefail

cargo fix
cargo fmt
