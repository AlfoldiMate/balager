#!/usr/bin/env bash
# Local development: build the client and run the API server (also serving
# the static client) on http://localhost:3000.
# Requires DATABASE_URL pointing at a Postgres database.
set -euo pipefail
cd "$(dirname "$0")/.."
bash scripts/build-client.sh
exec cargo run
