#!/usr/bin/env bash
# Vercel buildCommand. The WASM client is prebuilt locally by
# scripts/build-client.sh and committed (public/ + shell/), because the
# dioxus CLI's prebuilt binary needs a newer glibc than Vercel's build image
# provides. This script only validates the committed bundle; @vercel/rust
# compiles the api/main.rs function separately.
set -euo pipefail

test -f public/wasm/main.js || { echo "public/wasm/main.js missing — run scripts/build-client.sh locally and commit public/ + shell/"; exit 1; }
test -f shell/index.html || { echo "shell/index.html missing — run scripts/build-client.sh locally and commit it"; exit 1; }
test -f public/styles.css || { echo "public/styles.css missing — run scripts/build-client.sh locally and commit it"; exit 1; }
echo "Using committed client bundle ($(du -sh public | cut -f1))"
