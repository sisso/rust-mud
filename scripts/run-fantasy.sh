#!/bin/bash

set -euo pipefail

rm -f /tmp/mud_*.snapshot

cargo run data/fantasy
