#!/usr/bin/env bash
# Build the WASM client into ./public and the SSR shell into ./shell.
# Run this locally and COMMIT the result whenever client code changes —
# Vercel's build image cannot run dx (glibc too old for the prebuilt binary),
# so deployments use the committed bundle.
set -euo pipefail
cd "$(dirname "$0")/.."

# dx 0.7.3's release asset finalisation is flaky ("Failed to rename output
# file"); retry, then reconstruct what the rename race lost.
for attempt in 1 2 3 4 5; do
  if dx bundle --platform web --release --no-default-features --features web; then
    break
  fi
  echo "dx bundle attempt $attempt failed; retrying"
done

PUB=target/dx/main/release/web/public
test -d "$PUB/assets" || { echo "client bundle missing"; exit 1; }

# The loader module: newest hashed copy doubles as wasm/main.js.
MAIN_JS=$(/bin/ls -t "$PUB"/assets/main-dxh*.js | head -1)
mkdir -p "$PUB/wasm"
cp "$MAIN_JS" "$PUB/wasm/main.js"
HASHED_WASM=$(grep -o 'assets/main_bg-[a-z0-9]*\.wasm' "$PUB/wasm/main.js" | head -1 || true)
if [ -n "$HASHED_WASM" ] && [ ! -f "$PUB/$HASHED_WASM" ] && [ -f "$PUB/wasm/main_bg.wasm" ]; then
  cp "$PUB/wasm/main_bg.wasm" "$PUB/$HASHED_WASM"
fi
if [ ! -d "$PUB/wasm/snippets" ] && [ -d target/dx/main/debug/web/public/wasm/snippets ]; then
  cp -R target/dx/main/debug/web/public/wasm/snippets "$PUB/wasm/snippets"
fi
test -d "$PUB/wasm/snippets" || { echo "wasm snippets missing — run: dx build --platform web --no-default-features --features web"; exit 1; }

rm -rf public shell
mkdir -p public shell
cp -R "$PUB/." public/
# Prune stale hashed wasm/js copies accumulated by earlier dx runs; only the
# pair referenced by the loader is needed.
KEEP_WASM=$(basename "${HASHED_WASM:-}")
for f in public/assets/main_bg-dxh*.wasm; do
  [ "$(basename "$f")" = "$KEEP_WASM" ] || rm -f "$f"
done
KEEP_JS=$(basename "$MAIN_JS")
for f in public/assets/main-dxh*.js; do
  [ "$(basename "$f")" = "$KEEP_JS" ] || rm -f "$f"
done
cp -f assets/manifest.json assets/icon.png public/
cp -f assets/styles.css public/styles.css
mv public/index.html shell/index.html
# Copies for the serverless function (only shell/** is bundled into it):
# the SSR-injected hashed loader name varies per server build, so the
# function aliases such requests to these files.
cp public/wasm/main.js shell/loader.js
cp public/assets/main_bg-dxh*.wasm shell/main_bg.wasm
echo "Client bundle ready: public/ (static) + shell/ (SSR shell). Commit both."
