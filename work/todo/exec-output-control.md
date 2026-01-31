# Exec Output Control

Control how exec tag output is rendered inside `<hick:file>` blocks, analogous to RMarkdown/Quarto chunk options like `echo`, `eval`, `include`, and `output`.

## Motivation

Currently, every `<hick:exec>` inside a `<hick:file>` renders the full container transcript (all commands and all outputs). Authors need finer control:

- Show the command but hide its output (documentation of steps)
- Show the output but hide the command (clean result display)
- Hide both (exec runs for side effects only, e.g. setup)
- Show both (current default behavior = full transcript)

## Design

### New Attributes on `<hick:exec>` Inside Files

```xml
<hick:file path="guide.md">
<!-- Show command and output (default, current behavior) -->
<hick:exec container="demo">echo hello</hick:exec>

<!-- Show command only, hide output -->
<hick:exec container="demo" show="command">make build</hick:exec>

<!-- Show output only, hide command -->
<hick:exec container="demo" show="output">cat /out/result.txt</hick:exec>

<!-- Run but show nothing (side-effect only) -->
<hick:exec container="demo" show="none">rm -rf /tmp/cache</hick:exec>
</hick:file>
```

### Attribute: `show`

| Value | Commands shown | Output shown |
|-------|---------------|--------------|
| `all` (default) | Yes (`$ cmd`) | Yes |
| `command` | Yes (`$ cmd`) | No |
| `output` | No | Yes |
| `none` | No | No |

The `show` attribute only affects rendering inside `<hick:file>`. Top-level execs (outside files) are unaffected — they always execute and their output is always captured into the container transcript.

### Why Not Separate `echo`/`output` Booleans?

A single `show` attribute with four values is simpler than two boolean attributes (`echo="true" output="false"`). It avoids the question of what `echo="false" output="false"` means (same as `show="none"`), and it reads clearly in XML. If future needs demand finer control, `show` can be extended to a comma-separated list (`show="command,output"`) without breaking existing documents.

## Implementation

### Step 1: Parse the `show` Attribute

**File:** `crates/hick/src/lib.rs`

Add a helper enum and parser:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum ExecShow {
    All,
    Command,
    Output,
    None,
}

fn parse_exec_show(tag: &HickTag) -> ExecShow {
    match tag_attr(tag, "show").as_deref() {
        Some("command") => ExecShow::Command,
        Some("output") => ExecShow::Output,
        Some("none") => ExecShow::None,
        _ => ExecShow::All, // default
    }
}
```

### Step 2: Per-Exec Transcript Building

Currently transcripts are accumulated per-container (all execs concatenated). To support per-exec `show` control, we need per-exec transcript entries that can be selectively rendered.

**File:** `crates/hick/src/executor.rs`

Change the transcript structure from `HashMap<String, String>` to track individual exec entries:

```rust
#[derive(Debug, Clone)]
pub struct ExecTranscriptEntry {
    pub commands: Vec<String>,  // individual command lines
    pub output: String,         // captured output
}

// Change: transcripts now maps container -> Vec<ExecTranscriptEntry>
transcripts: HashMap<String, Vec<ExecTranscriptEntry>>,
```

At the end of `execute()`, push an entry instead of appending to a string:

```rust
let entry = ExecTranscriptEntry {
    commands: cmd_lines.clone(),
    output: result.clone(),
};
self.transcripts
    .entry(name.to_string())
    .or_default()
    .push(entry);
```

Add a convenience method for the full transcript (backwards compat):

```rust
pub fn full_transcript(&self, container: &str) -> String {
    let entries = self.transcripts.get(container).map(|v| v.as_slice()).unwrap_or(&[]);
    let mut out = String::new();
    for entry in entries {
        for cmd in &entry.commands {
            out.push_str(&format!("$ {cmd}\n"));
        }
        if !entry.output.is_empty() {
            out.push_str(&entry.output);
            out.push('\n');
        }
    }
    out
}
```

### Step 3: Render with `show` Control in `run_pipeline_live`

**File:** `crates/hick/src/lib.rs`

In the file-output processing section, when encountering an exec tag inside a file:

```rust
HickNode::Tag(child_tag) if child_tag.name == "exec" => {
    let container_name = tag_attr(child_tag, "container").unwrap_or_default();
    let show = parse_exec_show(child_tag);

    if show == ExecShow::None {
        // Side-effect only, render nothing
    } else if let Some(transcript) = transcripts.get(&container_name) {
        let rendered = render_transcript(transcript, show);
        file_ip.add(Arc::new(StringNode::new(rendered)));
    } else {
        file_ip.add(Arc::new(StringNode::new(
            format!("[no transcript for container '{container_name}']"),
        )));
    }
}
```

Add the rendering function:

```rust
fn render_transcript(entries: &[ExecTranscriptEntry], show: ExecShow) -> String {
    let mut out = String::new();
    for entry in entries {
        if matches!(show, ExecShow::All | ExecShow::Command) {
            for cmd in &entry.commands {
                out.push_str(&format!("$ {cmd}\n"));
            }
        }
        if matches!(show, ExecShow::All | ExecShow::Output) && !entry.output.is_empty() {
            out.push_str(&entry.output);
            out.push('\n');
        }
    }
    out
}
```

### Step 4: Dry-Run Transcript Entries

**File:** `crates/hick/src/lib.rs`

In `run_pipeline_with_authority()`, change the dry-run transcript builder to produce the same `Vec<ExecTranscriptEntry>` structure, with empty `output` fields:

```rust
let mut transcripts: HashMap<String, Vec<ExecTranscriptEntry>> = HashMap::new();
for (_, doc) in &documents {
    let flow_dag = dag::build_dag(doc).unwrap();
    for exec_id in flow_dag.topological_order() {
        let info = flow_dag.execs.iter().find(|e| e.id == exec_id).unwrap();
        let commands: Vec<String> = info.command.trim().lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        transcripts.entry(info.container.clone()).or_default().push(
            ExecTranscriptEntry { commands, output: String::new() }
        );
    }
}
```

Then use the same `render_transcript()` function in the file-output processing.

### Step 5: Update Tests

**File:** `crates/hick/tests/pipeline_tests.rs`

```rust
#[tokio::test]
async fn test_exec_show_command_only() {
    let src = hick_doc(
        r#"<hick:container name="demo" image="alpine" />
<hick:exec container="demo">echo setup</hick:exec>
<hick:file path="out.txt"><hick:exec container="demo" show="command">echo setup</hick:exec></hick:file>"#,
    );
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(content.contains("$ echo setup"), "should show command");
    // In dry-run, output is always empty, so this just verifies command is present
}

#[tokio::test]
async fn test_exec_show_none() {
    let src = hick_doc(
        r#"<hick:container name="demo" image="alpine" />
<hick:exec container="demo">echo setup</hick:exec>
<hick:file path="out.txt">before<hick:exec container="demo" show="none">echo setup</hick:exec>after</hick:file>"#,
    );
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert_eq!(content, "beforeafter", "show=none should render nothing");
}
```

## Files Modified

| File | Change |
|------|--------|
| `crates/hick/src/executor.rs` | `ExecTranscriptEntry` struct, structured transcript accumulation, `full_transcript()` method |
| `crates/hick/src/lib.rs` | `ExecShow` enum, `parse_exec_show()`, `render_transcript()`, updated file-output processing in both pipelines |
| `crates/hick/tests/pipeline_tests.rs` | Tests for each `show` value |

## Migration

No breaking changes. Existing documents without `show` attributes get `ExecShow::All` (current behavior).
