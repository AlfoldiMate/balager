#!/usr/bin/env bash
# Local development: build the client into ./public and run the API server
# (which also serves the static client) on http://localhost:3000.
# Requires DATABASE_URL pointing at a Postgres database.
set -euo pipefail
cd "$(dirname "$0")/.."

for attempt in 1 2 3 4 5; do
  if dx bundle --platform web --release --no-default-features --features web; then
    break
  fi
  echo "dx bundle attempt $attempt failed; retrying"
done

PUB=target/dx/main/release/web/public
HASHED_WASM=$(grep -o 'assets/main_bg-[a-z0-9]*\.wasm' "$PUB/wasm/main.js" | head -1 || true)
if [ -n "$HASHED_WASM" ] && [ ! -f "$PUB/$HASHED_WASM" ]; then
  cp "$PUB/wasm/main_bg.wasm" "$PUB/$HASHED_WASM"
fi

rm -rf public
mkdir -p public
cp -R "$PUB/." public/
cp -f assets/manifest.json assets/icon.png public/
cp -f assets/styles.css public/styles.css
# Keep the SSR shell out of the CDN static dir so "/" is server-rendered.
mkdir -p shell
mv public/index.html shell/index.html
# The dx bundle omits the unhashed loader/snippets the shell references.
MAIN_JS=$(/bin/ls -t public/assets/main-dxh*.js | head -1)
mkdir -p public/wasm
cp "$MAIN_JS" public/wasm/main.js
if [ ! -d public/wasm/snippets ] && [ -d target/dx/main/debug/web/public/wasm/snippets ]; then
  cp -R target/dx/main/debug/web/public/wasm/snippets public/wasm/snippets
fi

exec cargo run
