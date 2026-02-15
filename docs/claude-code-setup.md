# Sandboxed Command Execution with Claude Code

Run all shell commands in WASM Linux containers instead of directly on the host
machine. This provides security isolation and reproducibility for AI-assisted
development.

---

## Overview

By default, Claude Code runs shell commands directly on the dev machine via its
Bash tool. With hick-mcp's `hick_exec` tool, you can sandbox all command
execution inside WASM Linux containers while still giving the container direct
access to your project files.

The containers are powered by container2wasm (c2w), which runs a full Linux
kernel emulator inside WebAssembly. Your working directory is mounted at
`/workspace` inside the container, so commands operate on your real project
files.

Key benefits:
- **Security**: Commands run in an isolated environment with no access to the
  host system beyond the mounted working directory
- **Reproducibility**: Containers start from known images (e.g. Alpine Linux)
- **Persistence**: Containers persist across calls, so installed packages and
  environment variables are preserved within a session

---

## Prerequisites

1. **Build hick-mcp** from this repository:
   ```bash
   cargo build --release -p hick-mcp
   ```

2. **Prepare WASM container images** using c2w (container2wasm). See
   [Preparing Container Images](#preparing-container-images) below.

3. **Install Claude Code** (if not already installed).

---

## Configuration

### 1. Configure MCP Server

Add hick-mcp to your Claude Code MCP configuration (`.mcp.json` in your project
root or `~/.claude/mcp.json` for global config):

```json
{
  "mcpServers": {
    "hick": {
      "command": "/path/to/hick-mcp",
      "args": ["--images-dir", "/path/to/wasm-images"]
    }
  }
}
```

Optional arguments:
- `--workdir /path/to/project` -- Working directory to mount (defaults to CWD)
- `--default-image alpine` -- Default container image name (defaults to
  "alpine")

### 2. Configure Claude Code Permissions

To enforce that all commands go through the sandbox, deny the Bash tool in your
project's `.claude/settings.json`:

```json
{
  "permissions": {
    "deny": ["Bash"]
  }
}
```

### 3. Add CLAUDE.md Instructions

Tell Claude to use `hick_exec` for all command execution:

```markdown
## Command Execution

All commands run in WASM Linux containers via the hick_exec MCP tool.
The project directory is mounted at /workspace inside the container.
Do not attempt to use the Bash tool -- it is denied.

Example: to run tests, use hick_exec with command "cargo test".
```

---

## How It Works

1. **First call**: The container boots from the configured WASM image (~2-5s
   for Alpine). Your working directory is mounted at `/workspace`.

2. **Command execution**: Each `hick_exec` call runs `cd /workspace && <command>`
   in the container and returns stdout/stderr.

3. **Session persistence**: The container stays running between calls. Any
   packages you install (e.g. `apk add git`) persist for subsequent commands.

4. **Multiple containers**: Use the `container` parameter to run commands in
   separate isolated environments (e.g. one for building, one for testing).

---

## Preparing Container Images

Convert Docker images to WASM using container2wasm (c2w):

```bash
# Install c2w
go install github.com/aspect-build/container2wasm/cmd/c2w@latest

# Convert Alpine Linux
c2w alpine:3.19 alpine.wasm

# Convert Ubuntu
c2w ubuntu:22.04 ubuntu.wasm

# Move to your images directory
mkdir -p ~/wasm-images
mv alpine.wasm ubuntu.wasm ~/wasm-images/
```

Image naming: The `--default-image` flag and `image` parameter in `hick_exec`
reference images by their base name (without `.wasm`). Place images in the
directory specified by `--images-dir`.

---

## Tool Reference

### hick_exec

Run a shell command in a sandboxed WASM Linux container.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `command` | String | yes | -- | Shell command to run |
| `image` | String | no | server default | Container image name |
| `container` | String | no | "workspace" | Container name for session persistence |
| `timeout` | Integer | no | 120 | Timeout in seconds |

**Example:**

```json
{
  "tool": "hick_exec",
  "arguments": {
    "command": "ls -la",
    "timeout": 30
  }
}
```

---

## Troubleshooting

### Container fails to boot

- Verify the WASM image exists: `ls /path/to/wasm-images/alpine.wasm`
- Check hick-mcp stderr logs for boot timeout messages
- Ensure the image was converted with a compatible version of c2w

### /workspace is empty

The working directory mount relies on WASI preopened directories. If the c2w
emulator does not support runtime-mounted directories, the `/workspace`
directory may be empty. In this case:

1. Check hick-mcp logs for preopened dir warnings
2. Try an image built with explicit `--mapdir` support

### Commands time out

The default timeout is 120 seconds. For long-running commands (e.g.
compilation), increase the timeout:

```json
{
  "tool": "hick_exec",
  "arguments": {
    "command": "cargo build --release",
    "timeout": 600
  }
}
```

### Permission errors

The container runs as root by default. If you see permission errors on
`/workspace` files, it may be due to filesystem permission mapping. Check
whether the files are readable in the container:

```json
{
  "tool": "hick_exec",
  "arguments": {
    "command": "ls -la /workspace"
  }
}
```
