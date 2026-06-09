#!/usr/bin/env bash
# Vercel buildCommand: compile the WASM client into ./public.
# (The API function itself is compiled separately by @vercel/rust.)
set -euo pipefail

# Rust toolchain for the build container.
if ! command -v cargo >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
fi
# shellcheck disable=SC1091
source "$HOME/.cargo/env" 2>/dev/null || true
rustup target add wasm32-unknown-unknown

# Dioxus CLI (prebuilt binary via cargo-binstall; falls back to cargo install).
if ! command -v dx >/dev/null 2>&1; then
  if ! command -v cargo-binstall >/dev/null 2>&1; then
    curl -L --proto '=https' --tlsv1.2 -sSf \
      https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
  fi
  cargo binstall dioxus-cli@0.7.3 --no-confirm || cargo install dioxus-cli@0.7.3 --locked
fi

# dx 0.7.3's release asset finalisation is flaky ("Failed to rename output
# file"); retry, then reconstruct the loader-referenced hashed wasm if the
# rename race lost it (it is byte-identical to wasm/main_bg.wasm).
ok=0
for attempt in 1 2 3 4 5; do
  if dx bundle --platform web --release --no-default-features --features web; then
    ok=1
    break
  fi
  echo "dx bundle attempt $attempt failed; retrying"
done

PUB=target/dx/main/release/web/public
test -f "$PUB/wasm/main.js" || { echo "client bundle missing"; exit 1; }
HASHED_WASM=$(grep -o 'assets/main_bg-[a-z0-9]*\.wasm' "$PUB/wasm/main.js" | head -1 || true)
if [ -n "$HASHED_WASM" ] && [ ! -f "$PUB/$HASHED_WASM" ]; then
  cp "$PUB/wasm/main_bg.wasm" "$PUB/$HASHED_WASM"
  echo "reconstructed $HASHED_WASM"
fi
HASHED_CSS=$(ls "$PUB"/assets/styles-dxh*.css 2>/dev/null | head -1 || true)
if [ -z "$HASHED_CSS" ] && [ "$ok" = "0" ]; then
  echo "stylesheet missing from bundle after retries"
  exit 1
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
echo "Client built into ./public"
