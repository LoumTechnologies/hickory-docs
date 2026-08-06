# Hickory Docs — mobile (Tauri v2)

Tauri v2 shell wrapping the **same** React+TS app as `apps/web`. There is no
mobile UI code here: `tauri.conf.json` points `frontendDist` at
`../../web/dist` and `devUrl` at the web dev server, so web, iOS, and Android
are one codebase (per `docs/specs/freeform/architecture.md`).

- Bundle id: `com.loumtechnologies.hickorydocs`
- Rust crate: `src-tauri/` — deliberately **outside** the root Cargo workspace
  (`[workspace]` empty table in its `Cargo.toml`) so mobile SDK requirements
  never break `cargo check --workspace` at the repo root.
- Capabilities: `src-tauri/capabilities/default.json` (core defaults only).

## Prerequisites (all platforms)

```sh
cargo install tauri-cli --version '^2'   # provides `cargo tauri`
cd apps/web && npm install               # frontend deps
```

Desktop dev (Linux needs `webkit2gtk-4.1` + `libappindicator` dev packages):

```sh
cd apps/mobile/src-tauri
cargo tauri dev        # starts apps/web's Vite dev server, opens native window
cargo tauri build      # bundles with apps/web/dist
```

To demo without the Rust backend, run the web app in mock mode first:
`VITE_MOCK=1` is picked up by Vite (`npm run dev:mock` in `apps/web`).

## iOS (macOS only)

Requires Xcode (with iOS SDK + simulators) and:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
cd apps/mobile/src-tauri
cargo tauri ios init   # generates gen/apple (Xcode project) — run once
cargo tauri ios dev    # simulator; pass a device name to target one
cargo tauri ios build  # .ipa (configure signing team in gen/apple first)
```

`cargo tauri ios init` was **not** run in this repo: it requires Xcode and can
only run on macOS. The generated `gen/apple/` directory is machine-produced;
commit it after first init. Signing: set your Apple development team in the
generated Xcode project (or `TAURI_APPLE_DEVELOPMENT_TEAM` env var).

## Android

Requires the Android SDK + NDK and JDK 17. Set env vars first:

```sh
export ANDROID_HOME="$HOME/Android/Sdk"
export NDK_HOME="$ANDROID_HOME/ndk/<version>"
export JAVA_HOME=/usr/lib/jvm/java-17-openjdk   # adjust per OS
rustup target add aarch64-linux-android armv7-linux-androideabi \
  i686-linux-android x86_64-linux-android
cd apps/mobile/src-tauri
cargo tauri android init   # generates gen/android (Gradle project) — run once
cargo tauri android dev    # emulator/device
cargo tauri android build  # .aab/.apk
```

`cargo tauri android init` was **not** run in this repo: no Android SDK/NDK is
present on the machine that scaffolded it (checked: `ANDROID_HOME` unset, no
`cargo-tauri`). Commit `gen/android/` after first init.

## Verifying the crate without mobile SDKs

```sh
cd apps/mobile/src-tauri
cargo check   # host-target check; needs webkit2gtk-4.1 on Linux, nothing mobile
```
