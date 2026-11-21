#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

case "$(rustc -vV | sed -n 's/^host: //p')" in
  x86_64-pc-windows-msvc) runtime=win-x64 ;;
  aarch64-pc-windows-msvc) runtime=win-arm64 ;;
  *) echo 'Run just simple-build windows on Windows with the MSVC Rust toolchain and .NET 10 SDK.' >&2; exit 2 ;;
esac

build=target/simple-windows
if [ ! -f apps/web/node_modules/vite/bin/vite.js ] || [ apps/web/package-lock.json -nt apps/web/node_modules ]; then
  npm --prefix apps/web ci
fi
(cd apps/web && node node_modules/vite/bin/vite.js build --config vite.simple.config.ts \
  --outDir ../../target/simple-windows/web --emptyOutDir)
cargo build --locked --target-dir target -p hickory-cli --example simple_dev
dotnet publish apps/windows-simple/HickorySimple.csproj --configuration Release \
  --runtime "$runtime" --self-contained --artifacts-path "$build/dotnet" --output "$build/app"
cp target/debug/examples/simple_dev.exe "$build/app/hickory-simple-host.exe"
rm -rf "$build/app/web"
cp -R "$build/web" "$build/app/web"
