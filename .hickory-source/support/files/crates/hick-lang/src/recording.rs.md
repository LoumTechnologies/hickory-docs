<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/src/recording.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Rebuild the program from its Markdown source](../../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md)

Captured SHA-256: `3148ffa3b7baf16e6279164e70bd51852555b9798aeb590beb34dceec60ee5f4`.

<book:entry path="crates/hick-lang/src/recording.rs">
use sha2::{Digest, Sha256};

/// Compute the cache key (SHA-256 hex) for an exec step.
///
/// **The key covers the cell's inputs, not just its text.** A recording
/// answers "what did this cell produce last time", and that question is only
/// well posed once the answer is pinned to everything that could change the
/// output. Four of those are properties of the cell itself — the image, the
/// capabilities it was granted, the command, and which secrets were in scope.
/// The other two are what it read:
///
/// * `input_digest` — the contents of the volumes mounted into this cell, from
///   [`inputs_digest`]. This is what makes a `<hick:file>` product an input:
///   the document assembles `analysis.py`, the file is seeded into a volume,
///   and the cell that runs it re-executes when it changes. Keyed on the
///   command alone, editing that file would serve the old recording and the
///   document would report numbers produced by code it no longer contains.
/// * `upstream_keys` — the keys of this cell's predecessors in the flow DAG.
///   Chaining them makes invalidation transitive without hashing anything
///   twice: if an upstream cell's inputs changed, its key changed, so every
///   cell downstream of it has a different key too. Container state, volume
///   flow, and copy/paste edges are all carried by this one term, because the
///   DAG already models each of them as an edge.
///
/// Order matters for `upstream_keys`; callers pass them in the DAG's
/// deterministic predecessor order so the same graph always yields the same
/// key.
pub fn exec_cache_key(
    container_image: &str,
    capabilities_canonical: &str,
    command: &str,
    secret_names: &[&str],
    input_digest: &str,
    upstream_keys: &[&str],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"image:");
    hasher.update(container_image.as_bytes());
    hasher.update(b"\ncaps:");
    hasher.update(capabilities_canonical.as_bytes());
    hasher.update(b"\ncmd:");
    hasher.update(command.trim().as_bytes());
    for s in secret_names {
        hasher.update(b"\nsecret:");
        hasher.update(s.as_bytes());
    }
    hasher.update(b"\ninputs:");
    hasher.update(input_digest.as_bytes());
    for k in upstream_keys {
        hasher.update(b"\nupstream:");
        hasher.update(k.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// Digest the files a cell can read, as `(path, bytes)` pairs.
///
/// Sorted by path before hashing, because the order files come out of a volume
/// is not a property of the volume — an unsorted digest would make the same
/// inputs produce different keys on different runs and no cell would ever hit
/// its recording. The length of each path and body is folded in so that
/// `("ab", "c")` and `("a", "bc")` cannot collide.
///
/// An empty input set has its own digest rather than the empty string, so
/// "this cell reads nothing" is a statement the key records rather than an
/// absence it cannot distinguish from "inputs not computed".
pub fn inputs_digest(files: &[(String, Vec<u8>)]) -> String {
    let mut sorted: Vec<&(String, Vec<u8>)> = files.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    hasher.update(b"inputs-v1:");
    for (path, body) in sorted {
        hasher.update((path.len() as u64).to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update((body.len() as u64).to_le_bytes());
        hasher.update(body);
    }
    format!("{:x}", hasher.finalize())
}

/// Compute the cache key (SHA-256 hex) for a `<hick:agent>` cell.
///
/// **The prompt and the model are both in the key, and both have to be.** A
/// recording answers the question "what did this cell produce last time"; for
/// an agent cell the question is only well posed once you say *what was asked*
/// and *who was asked*. Key on the prompt alone and editing the prompt serves
/// the old recording — freeze would then verify a claim the document no longer
/// makes. Key on the model alone and the same failure happens the other way.
///
/// Deliberately NOT in the key: `max-turns`. It bounds how hard the cell may
/// try, not what it was asked, and a recording made under a larger budget is
/// still an honest answer to the same question.
///
/// The domain separator differs from [`exec_cache_key`]'s (`agent:` vs
/// `image:`), so an agent cell and an exec cell can never collide on a key
/// even if their text matched byte for byte.
pub fn agent_cache_key(model: &str, prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"agent:");
    hasher.update(b"\nmodel:");
    hasher.update(model.as_bytes());
    hasher.update(b"\nprompt:");
    hasher.update(prompt.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Shared transcript rendering. Formatting bytes are synthetic, not source maps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecShow {
    All,
    Command,
    Output,
    None,
}
pub fn render_transcript_entry(commands: &[String], output: &str, show: ExecShow) -> String {
    let mut text = String::new();
    if matches!(show, ExecShow::All | ExecShow::Command) {
        for command in commands {
            let source = crate::notebook::source_command(command);
            for (i, line) in source.as_deref().unwrap_or(command).lines().enumerate() {
                text.push_str(if i == 0 { "$ " } else { "  " });
                text.push_str(line);
                text.push('\n');
            }
        }
    }
    if matches!(show, ExecShow::All | ExecShow::Output) && !output.is_empty() {
        text.push_str(&crate::notebook::render_output(output));
        text.push('\n');
    }
    text
}
</book:entry>

</book:doc>
