# emu-x86

Implements three `EmulatorProvider` variants for x86_64, all using the `blink`
WASM emulator.

| Provider | Target | Notes |
|---|---|---|
| `BlinkProvider` | Linux ELF x86_64 | Direct execution |
| `BlinkMachoProvider` | macOS Mach-O x86_64 | Adds a Mach-O loader shim and XNU-to-Linux BSD syscall translation table (`syscall_map.txt`) |
| `BlinkPeProvider` | Windows PE x86_64 | Adds a PE loader shim and Win32 API–to–Linux syscall map (`api_map.txt`); complex Win32 API use requires VM/Wine |
