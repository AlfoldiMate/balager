#!/usr/bin/env bash
# Local development: build the client and run the API server (also serving
# the static client) on http://localhost:3000.
# Requires DATABASE_URL pointing at a Postgres database.
set -euo pipefail
cd "$(dirname "$0")/.."
# Server-fn route hashes default to xxh64(CARGO_MANIFEST_DIR + module path),
# which differs between this machine and Vercel's build dir; a fixed override
# keeps client and server in agreement. Must match build.env in vercel.json.
export SERVER_FN_OVERRIDE_KEY=balager
bash scripts/build-client.sh
exec cargo run
