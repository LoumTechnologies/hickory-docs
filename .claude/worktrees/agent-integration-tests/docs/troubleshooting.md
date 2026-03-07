# Hick Troubleshooting

This guide covers common errors, their causes, and how to resolve them.

---

## Table of Contents

1. [Parse Errors](#1-parse-errors)
2. [DAG Validation Errors](#2-dag-validation-errors)
3. [Container Errors](#3-container-errors)
4. [Secrets Errors](#4-secrets-errors)
5. [File Output Errors](#5-file-output-errors)
6. [Copy / Paste Issues](#6-copy--paste-issues)
7. [Fork Errors](#7-fork-errors)
8. [Network and Capability Errors](#8-network-and-capability-errors)
9. [Debugging Tips](#9-debugging-tips)

---

## 1. Parse Errors

### Missing root doc element

```
Error: parse error in docs/example.hick: missing root doc element (no xmlns for http://www.hickorydocs.com/1.0)
```

**Cause:** The file does not contain a root `<PREFIX:doc>` element with the
hick namespace declaration.

**Fix:** Ensure the file has a root element binding a prefix to the hick
namespace:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
  <!-- content -->
</hick:doc>
```

**Common mistakes:**

- Typo in the namespace URI (must be exactly
  `http://www.hickorydocs.com/1.0`)
- Using a default namespace (`xmlns="..."`) instead of a prefixed one
  (`xmlns:hick="..."`)
- The root element name must be `doc` (e.g., `<hick:doc>`, not
  `<hick:document>`)

### Unclosed tag

```
Error: unclosed tag <hick:exec> opened at line 5
```

**Cause:** An opening tag was never closed with the corresponding closing
tag.

**Fix:** Add the closing tag:

```xml
<!-- Wrong -->
<hick:exec container="demo">
echo hello

<!-- Right -->
<hick:exec container="demo">
echo hello
</hick:exec>
```

**Common mistakes:**

- Forgetting to close `exec`, `file`, `container`, `copy`, `cut`, `fork`,
  or `attenuate` tags
- Misspelling the closing tag name (e.g., `</hick:exce>` instead of
  `</hick:exec>`)
- Using the wrong prefix in the closing tag (e.g., `</h:exec>` when the
  document uses `hick:`)

### Unexpected closing tag

```
Error: line 8: unexpected closing tag </hick:doc>
```

**Cause:** A closing tag appeared where it wasn't expected -- usually
because a child tag was not properly closed first.

**Fix:** Check that all nested tags are closed in the correct order:

```xml
<!-- Wrong: exec not closed before doc -->
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo">
echo hello
</hick:doc>

<!-- Right -->
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo">
echo hello
</hick:exec>
</hick:doc>
```

### Empty tag name

```
Error: line 3: empty tag name after <hick:
```

**Cause:** A tag starts with the prefix but has no name after the colon.

**Fix:** Ensure every tag has a name: `<hick:exec>`, not `<hick:>`.

### Attribute value must be quoted

```
Error: line 4: attribute value must be quoted
```

**Cause:** An attribute value is not wrapped in quotes.

**Fix:** Use single or double quotes around all attribute values:

```xml
<!-- Wrong -->
<hick:exec container=demo>

<!-- Right -->
<hick:exec container="demo">

<!-- Also right -->
<hick:exec container='demo'>
```

### Unterminated attribute value

```
Error: line 4: unterminated attribute value
```

**Cause:** An attribute value's opening quote is never closed.

**Fix:** Ensure quotes are balanced:

```xml
<!-- Wrong -->
<hick:exec container="demo>

<!-- Right -->
<hick:exec container="demo">
```

### Expected '=' after attribute

```
Error: line 4: expected '=' after attribute 'container'
```

**Cause:** An attribute name is not followed by `=`.

**Fix:** Use `name="value"` syntax for all attributes.

---

## 2. DAG Validation Errors

### Cycle detected

```
Error: DAG validation failed in docs/example.hick: cycle detected involving exec[2]
```

**Cause:** The dependency graph contains a circular dependency. This usually
happens when two containers or volumes create a mutual dependency.

**Fix:** Review the dependency structure. Common causes:

- Container A writes to volume X, Container B reads from X and writes to
  volume Y, Container A reads from Y (circular volume dependency)
- A copy/paste chain that references itself

**Debug:** Use `--verbose` to see the exec nodes and their dependencies.
The error identifies the exec index involved in the cycle.

### Broken volume flow

```
Error: broken flow: exec 3 reads volume 'shared' but no exec writes to it
```

**Cause:** An `exec` mounts a volume as a reader, but no prior `exec`
mounts it as a writer.

**Fix:** Ensure the volume is mounted by at least one `exec` before any
readers:

```xml
<hick:volume name="shared" />

<!-- This exec writes first (first mounter = writer) -->
<hick:exec container="producer" mount="shared:/output">
echo data > /output/file.txt
</hick:exec>

<!-- This exec reads (subsequent mounter = reader) -->
<hick:exec container="consumer" mount="shared:/input">
cat /input/file.txt
</hick:exec>
```

### Broken paste flow

```
Error: broken flow: exec 2 uses paste select '#version' but no copy with that id exists
```

**Cause:** A `paste` tag references a `copy` or `cut` block that doesn't
exist.

**Fix:** Ensure the referenced id exists as either a `copy` or `cut` block:

```xml
<!-- Define the content block first -->
<hick:copy id="version">1.0.0</hick:copy>

<!-- Then reference it -->
<hick:paste select="#version" />
```

**Common mistakes:**

- Typo in the id (e.g., `#versoin` vs `#version`)
- Forgetting the `#` prefix in the selector
- The copy/cut block is defined in a different file that isn't part of the
  pipeline run

---

## 3. Container Errors

### Failed to load WASM image

```
Error: failed to load WASM image: crates/c2w/converted_images/alpine.wasm
```

**Cause:** The `.wasm` file for the container image doesn't exist in the
images directory.

**Fix:**

1. Check that the images directory contains the required `.wasm` file
2. Use `--images-dir` to point to the correct directory
3. Convert the OCI image to WASM using the container2wasm tool

**Image resolution order:**

1. `{images_dir}/{base_name}.wasm` (exact file)
2. `{images_dir}/{base_name}/` (directory, latest `.wasm` by sort)

The image tag (e.g., `:3.20` in `alpine:3.20`) is stripped during
resolution -- only the base name is used.

### Container failed to boot

```
Error: container 'builder' failed to boot: timed out waiting for boot prompt
```

**Cause:** The container kernel didn't show a shell prompt within 120
seconds.

**Fix:**

- Ensure the WASM image is a valid container2wasm-converted image
- Check that the image boots to a BusyBox ash shell (the prompt detector
  looks for `/ # `)
- Try running the image manually with wasmtime to debug boot issues
- Check system resources -- WASM containers need memory and CPU to boot

### Container exited during boot

```
Error: container 'builder' failed to boot: container exited during boot
```

**Cause:** The container process terminated before completing boot.

**Fix:**

- The WASM image may be corrupted or incompatible
- Check that wasmtime is at a compatible version
- Try re-converting the OCI image to WASM

### Timed out waiting for command output

```
Error: timed out waiting for command output
```

**Cause:** A command didn't produce its end marker within 300 seconds.

**Fix:**

- The command may be hanging (e.g., waiting for input, stuck in a loop)
- Network requests may be blocked by capability rules, causing timeouts
- The command may genuinely take too long -- consider breaking it into
  smaller steps

### Container exited unexpectedly

```
Error: container exited unexpectedly
```

**Cause:** The container process died while a command was running.

**Fix:**

- The command may have caused a kernel panic or out-of-memory condition
- Check the command for issues (e.g., infinite memory allocation)
- Use `--verbose` to see the container output leading up to the exit

### Fork source container not found

```
Error: fork source container 'base' not found (not started yet?)
```

**Cause:** A fork references a source container that hasn't been started.

**Fix:** Ensure the source container has at least one `exec` that runs
before the fork. The fork replays the source container's command history, so
the source must be started and have executed commands first.

---

## 4. Secrets Errors

### Secret not found

```
Error: secret not found: github-token
```

**Cause:** The encrypted secret file doesn't exist in the secrets directory.

**Fix:**

1. Create the encrypted secret file:
   ```bash
   echo -n "your-secret-value" | age -r $(age-keygen -y ~/.config/hick/key.txt) \
     -o ~/.config/hick/secrets/github-token.age
   ```
2. Verify the file exists: `ls ~/.config/hick/secrets/github-token.age`
3. If using a custom secrets directory, pass `--secrets-dir`

### Decryption failed

```
Error: decryption failed: ...
```

**Cause:** The age identity file (private key) doesn't match the key used
to encrypt the secret.

**Fix:**

- Verify the key file path: `--key-file ~/.config/hick/key.txt`
- Re-encrypt the secret with the correct public key:
  ```bash
  age-keygen -y ~/.config/hick/key.txt  # get the public key
  echo -n "value" | age -r age1... -o secret.age
  ```
- If the key was rotated, re-encrypt all secrets with the new key

### Identity file error

```
Error: identity file error: ...
```

**Cause:** The age identity file is missing, malformed, or unreadable.

**Fix:**

- Check that `~/.config/hick/key.txt` exists and is readable
- Generate a new key: `age-keygen -o ~/.config/hick/key.txt`
- Check file permissions: `chmod 600 ~/.config/hick/key.txt`

### Invalid UTF-8 in secret

```
Error: invalid UTF-8 in secret: ...
```

**Cause:** The decrypted secret content is not valid UTF-8 text.

**Fix:** Secrets must be valid UTF-8 strings (they're injected as
environment variables). Re-encrypt the secret ensuring the plaintext is
valid UTF-8.

---

## 5. File Output Errors

### No file outputs produced

```
INFO No file outputs produced
```

**Cause:** The document contains no `<hick:file>` tags.

**This is not an error** -- it simply means no files will be written. The
document may only contain container definitions and exec steps for pipeline
purposes.

### Permission denied writing file

```
Error: Permission denied (os error 13)
```

**Cause:** Hick doesn't have permission to write to the output path.

**Fix:**

- Check directory permissions for the output path
- Ensure the path is relative (hick writes relative to the current working
  directory)
- Run from a directory where you have write access

### No transcript for container

If a file output contains:

```
[no transcript for container 'builder']
```

**Cause:** The `exec` tag inside a `file` references a container that has
no recorded transcript. This can happen if:

- The container name is misspelled
- The container's execs are defined in a different file not included in the
  pipeline run
- The container was never started (no `exec` for it at the top level)

**Fix:** Ensure the container name matches and that execs for the container
exist in the pipeline.

---

## 6. Copy / Paste Issues

### Paste selector not found

```
WARN Paste selector '#version' not found
```

**Cause:** A `paste` tag references an id that has no matching `copy` or
`cut` block.

**Fix:**

- Check the id spelling in both the `copy`/`cut` and `paste` tags
- Ensure the `#` prefix is present in the `select` attribute
- If using multiple files, ensure the file with the `copy`/`cut` block is
  included in the pipeline

### Paste produces empty content

**Cause:** The `copy` or `cut` block exists but contains empty text.

**Fix:** Check the content inside the `copy`/`cut` tags:

```xml
<!-- Empty content -->
<hick:copy id="version"></hick:copy>

<!-- Content with value -->
<hick:copy id="version">1.0.0</hick:copy>
```

### Cross-document paste not resolving

**Cause:** Copy/paste works across files only when they're processed in the
same pipeline run.

**Fix:** Pass all related files in a single invocation:

```bash
# Copy in setup.hick, paste in guide.hick
hick docs/setup.hick docs/guide.hick
```

---

## 7. Fork Errors

### Fork replaying commands fails

```
Error: failed to replay command in forked container 'analyzer': command text
```

**Cause:** A command from the source container's history failed when
replayed in the forked container.

**Fix:**

- The forked container has additional restrictions that prevent the command
  from succeeding (e.g., network denied but the command needs network
  access)
- Review the fork's capability restrictions to ensure they don't block
  commands that ran in the source

### Fork without source execs

If a fork target starts but has unexpected state, the source container may
not have been fully initialised.

**Fix:** Ensure the source container has `exec` tags that run before the
`fork` in document order. The DAG enforces this, but the source must have
at least one `exec`.

---

## 8. Network and Capability Errors

### Container can't reach network hosts

If a container fails to connect to a host (e.g., `wget` or `curl` times
out):

**Possible causes:**

1. **Deny-all rule active** -- if `<hick:deny network="*" />` is present,
   only explicitly allowed hosts are reachable
2. **Wrong host or port** -- the allow rule doesn't match the actual
   destination
3. **DNS resolution** -- DNS may need its own allow rule or network config

**Fix:** Check the container's capability rules:

```xml
<hick:container name="builder" image="alpine">
  <hick:allow network="github.com:443" />
  <hick:deny network="*" />
  <!-- Can only reach github.com on port 443, nothing else -->
</hick:container>
```

### File access denied in container

If a container can't read or write files at expected paths:

**Possible causes:**

1. File rules are defined but don't cover the target path
2. A `file-read` rule doesn't grant write access (write requires
   `file-write`)
3. The glob pattern doesn't match (e.g., `/output` without trailing `/*`
   only matches the exact path `/output`)

**Fix:** Review and adjust file rules:

```xml
<!-- Allow read AND write under /output/ -->
<hick:allow file-write="/output/*" />

<!-- Allow only read under /input/ -->
<hick:allow file-read="/input/*" />
```

---

## 9. Debugging Tips

### Enable verbose logging

```bash
hick -v docs/guide.hick
```

Verbose mode enables debug-level logging, showing:

- Parse results (number of top-level nodes per document)
- DAG validation results (number of exec nodes)
- Container start and boot events
- Command execution and output capture
- Token minting
- File output writing

### Use RUST_LOG for granular control

```bash
# All debug logging
RUST_LOG=debug hick docs/guide.hick

# Only hick crate debug logging
RUST_LOG=hick=debug hick docs/guide.hick

# Container crate debug logging
RUST_LOG=container=debug hick docs/guide.hick
```

### Start with dry-run

When developing a new `.hick` document, start with `--dry-run` to validate
the structure without waiting for containers:

```bash
hick --dry-run docs/guide.hick
```

This catches all parse errors and DAG validation issues immediately.

### Check DAG structure

Use verbose mode to see how many exec nodes and dependencies were detected:

```
INFO Parsed docs/guide.hick: 12 top-level nodes
INFO DAG validated for docs/guide.hick: 4 exec nodes
```

If the exec count is wrong, an `exec` tag may be malformed or missing.

### Inspect generated files

After a dry-run, check the generated files to verify the document structure:

```bash
hick --dry-run docs/guide.hick
cat docs/guide.md  # or whatever the file path is
```

### Common workflow for debugging

1. **Parse check:** Run with `--dry-run` to catch syntax errors
2. **DAG check:** Verbose dry-run to verify dependency graph
3. **Single file:** Test with one file before combining multiple
4. **Simple commands:** Start with `echo` commands before complex scripts
5. **Capabilities last:** Get the pipeline working without restrictions,
   then add `deny` rules

### Checking installed images

List available WASM images:

```bash
ls crates/c2w/converted_images/
# or
ls /path/to/your/images-dir/
```

Ensure each referenced image has a corresponding `.wasm` file or directory.

### Multi-file state sharing

When debugging cross-file copy/paste or state issues, remember:

- Files are processed in the order they're passed on the command line
- Copy/cut blocks from earlier files are available to later files
- All files share the same pipeline state

```bash
# Order matters for copy/paste resolution
hick docs/definitions.hick docs/guide.hick
```
