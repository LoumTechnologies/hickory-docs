<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Keep mobile builds independent from the desktop workspace

Keep mobile builds independent from the desktop workspace. This literal construction preserves build configuration exactly.

<mobile:entry path="apps/mobile/Cargo.toml">
[workspace]
resolver = "3"
members = ["core", "src-tauri"]

[profile.dev]
debug = "line-tables-only"
</mobile:entry>
</mobile:doc>
