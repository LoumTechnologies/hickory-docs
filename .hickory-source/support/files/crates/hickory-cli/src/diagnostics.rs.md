<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/diagnostics.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `7629b551c28bcc08a268cf5ba527fe43cb02f073415e582e931cbfda5210c3bf`.

<book:entry path="crates/hickory-cli/src/diagnostics.rs">
use super::*;

/// XML entities that reached a command verbatim.
///
/// hick's no-escaping invariant means `&lt;` inside an exec body is five
/// literal characters handed to the shell, not `<`. That is the correct and
/// deliberate behaviour — but it is also the single most likely mistake for
/// anyone who has ever written XML, and it fails far from its cause: the
/// shell reports `Syntax error: "&" unexpected` and says nothing about
/// escaping. (Written by an agent that made exactly this mistake while
/// dogfooding, and spent a cycle on the shell error before seeing it.)
///
/// A warning, never an error: a document may legitimately discuss entities.
/// Documents ABOUT hick use the `h:` prefix, so their examples are text and
/// never reach here.
pub fn escaping_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    const ENTITIES: &[(&str, char)] = &[
        ("&lt;", '<'),
        ("&gt;", '>'),
        ("&amp;", '&'),
        ("&quot;", '"'),
        ("&apos;", '\''),
    ];
    // Extensions where an entity is ordinary content rather than a mistake.
    // An HTML page full of `&amp;` is correct HTML, and warning about it
    // would teach people to ignore the warning that matters.
    const MARKUP: &[&str] = &[
        "html", "htm", "xml", "xhtml", "svg", "xsl", "xslt", "rss", "atom", "md", "markdown",
        "vue", "razor", "cshtml", "xaml", "plist", "resx",
    ];
    let mut out = Vec::new();
    for tag in doc.tags() {
        let (what, where_to) = match tag.name.as_str() {
            "exec" => ("exec body", "the shell receives those characters literally"),
            // A `hick:file` is where this mistake is worst. The shell at
            // least fails loudly and soon; a generated source file takes the
            // five literal characters, is written without complaint, and
            // fails later in a compiler that has never heard of hick. That
            // is a long way from the cause.
            "file" => {
                let path = tag.get_attribute("path").unwrap_or_default();
                let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
                if MARKUP.contains(&ext.as_str()) {
                    continue;
                }
                (
                    "file body",
                    "the generated file receives those characters literally",
                )
            }
            _ => continue,
        };
        let body = hick_lang::tag_text(tag);
        for (entity, literal) in ENTITIES {
            if body.contains(entity) {
                out.push(format!(
                    "line {}: {what} contains `{entity}` — hick does not \
                     unescape, so {where_to}. Write `{literal}` directly.",
                    tag.source_line
                ));
                break;
            }
        }
    }
    out
}

/// Diagrams whose claim nothing checks, and diagrams claiming a check that
/// does not exist.
///
/// A picture of a system is an assertion — "these are the layers, this one
/// never calls that one" — and the reason architecture diagrams are all
/// wrong is that nothing ever re-reads them. `asserts` is how a diagram names
/// the cells that prove it:
///
/// ```text
/// <hick:diagram renderer="mermaid" asserts="#no-back-edges">
/// ```
///
/// Two failure modes, and they need different words. A diagram naming an id
/// that is nowhere in the document is a BROKEN reference — the author meant
/// to be checked and is not, usually because a cell was renamed. A diagram
/// with no `asserts` at all is an UNVERIFIED drawing: legal, sometimes
/// deliberate (a sketch of something outside this repository), and worth
/// saying out loud exactly once so nobody mistakes it for a checked one.
///
/// Both stay warnings. Whether a picture needs proof is the author's call,
/// and a tool that refused to weave an unproven sketch would just teach
/// people to draw somewhere else.
pub fn diagram_assertion_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    let ids: std::collections::HashSet<&str> =
        doc.tags().filter_map(|t| t.get_attribute("id")).collect();
    let mut out = Vec::new();
    for tag in doc.tags().filter(|t| t.name == "diagram") {
        let line = tag.source_line;
        // A graph scene that does not parse weaves as its raw JSON — legal,
        // but the author should hear about it here, beside the assertion
        // warnings, rather than discover an unreadable fence later. A body
        // holding a `<hick:paste>` is checked after resolution instead, which
        // only the run can do.
        if tag.get_attribute("renderer") == Some(hick_literate::scene::GRAPH_RENDERER)
            && !tag.child_tags().any(|t| t.name == "paste")
        {
            match hick_literate::scene::parse_scene(&tag.text_content()) {
                Ok(scene) => {
                    for warning in hick_literate::scene::scene_warnings(&scene) {
                        out.push(format!("line {line}: {warning}"));
                    }
                }
                Err(err) => out.push(format!(
                    "line {line}: this graph diagram's body is not a scene \
                     ({err}), so it will weave as raw JSON instead of a \
                     picture. A scene is {{\"nodes\": …, \"edges\": …, \
                     \"layout\": …}} — see \
                     docs/specs/freeform/a-diagram-you-can-drag.md."
                )),
            }
        }
        let Some(asserts) = tag.get_attribute("asserts") else {
            out.push(format!(
                "line {line}: this diagram asserts nothing, so nothing will \
                 fail when it stops being true. Add `asserts=\"#id\"` naming \
                 the cell(s) that prove it, or leave it as a deliberate sketch."
            ));
            continue;
        };
        for selector in asserts.split_whitespace() {
            let Some(id) = selector.strip_prefix('#') else {
                out.push(format!(
                    "line {line}: `asserts` takes `#id` selectors, so `{selector}` \
                     matches nothing. Give the proving cell an `id` and name it here."
                ));
                continue;
            };
            if !ids.contains(id) {
                out.push(format!(
                    "line {line}: this diagram says it is proved by `#{id}`, but \
                     no tag in this document has that id — the check it names \
                     does not exist. Renaming a cell without renaming the \
                     reference is the usual cause."
                ));
            }
        }
    }
    out
}

/// Claims whose standing is missing, unknown, or unfalsifiable as written.
///
/// A `<hick:claim>` is an assertion ABOUT an assertion: nothing verifies that
/// Sam is an expert, and nothing ever will. What can be checked is whether the
/// marking is meaningful enough to be worth reading — which is what these
/// warnings are for. See `docs/specs/freeform/provenance-and-standing.md`.
///
/// All of these are **warnings, never errors**, following the same reasoning as
/// the diagram assertions: a tool that refused to weave an imperfectly marked
/// claim would only teach people to stop marking claims, and an unused marking
/// system is worse than none because it makes the marked subset look complete.
pub fn claim_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    let extra: Vec<String> = doc
        .frontmatter
        .as_ref()
        .map(|fm| fm.list("standings"))
        .unwrap_or_default();
    let allowed: Vec<&str> = hick_lang::STANDINGS
        .iter()
        .copied()
        .chain(extra.iter().map(String::as_str))
        .collect();
    let vocabulary = allowed.join(", ");

    let mut out = Vec::new();
    for tag in doc.tags().filter(|t| t.name == "claim") {
        let line = tag.source_line;

        if tag
            .get_attribute("by")
            .filter(|by| !by.is_empty())
            .is_none()
        {
            out.push(format!(
                "line {line}: this claim says nothing about who is making it. \
                 Add `by=\"name\"` — an unattributed claim is indistinguishable \
                 from the document's own prose, which is what marking it was \
                 supposed to prevent."
            ));
        }

        match tag.get_attribute("standing").filter(|s| !s.is_empty()) {
            None => out.push(format!(
                "line {line}: this claim declares no `standing`, so it says \
                 someone asserted something without saying on what footing. \
                 Use one of: {vocabulary}."
            )),
            Some(standing) if !allowed.contains(&standing) => out.push(format!(
                "line {line}: `standing=\"{standing}\"` is not a standing this \
                 document knows. Use one of: {vocabulary} — or add \"{standing}\" \
                 to a `standings:` list in this document's frontmatter if it is \
                 a distinction your notes really make."
            )),
            Some("expert")
                if tag
                    .get_attribute("scope")
                    .filter(|s| !s.is_empty())
                    .is_none() =>
            {
                out.push(format!(
                    "line {line}: this claims expertise without a `scope`. \
                     Expertise is never global, and an expert speaking outside \
                     their scope is exactly what this marking exists to make \
                     visible — add `scope=\"…\"` naming what they are expert in."
                ));
            }
            Some(_) => {}
        }
    }
    out
}

/// Mounts at absolute paths, which the local executor cannot honour.
///
/// Both executors map `mount="v:/out"` to `/out` UNDER the container workdir
/// — `LocalExecutor` to `<tmp>/<container>/out`, `CanopyExecutor` to
/// `/hickory-work/out`, and there is a test in the canopy crate
/// (`mount_dir_mirrors_local_executor_semantics`) asserting they agree. A
/// command that then writes to the ABSOLUTE `/out` addresses the real
/// filesystem root and fails with `cannot create /out/...: Directory
/// nonexistent`, an error naming neither the mount nor the cause.
///
/// So this is not a dev/prod divergence: such a document is broken on every
/// executor. It stays a warning rather than an error because only the
/// commands know which paths they use, and this checks the declaration.
pub fn absolute_mount_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    let mut out = Vec::new();
    for tag in doc.tags().filter(|t| t.name == "exec") {
        let Some(mount) = tag.get_attribute("mount") else {
            continue;
        };
        for spec in mount.split(',') {
            let Some((_vol, path)) = spec.split_once(':') else {
                continue;
            };
            let path = path.trim();
            if !path.starts_with('/') {
                continue;
            }
            // The spec `src:/project` is fine on its own — every executor
            // strips the slash and mounts under the workdir. What fails is a
            // COMMAND that then says `/project/...`, so warn only when one
            // does; a cell that addresses `project/...` relatively has done
            // nothing wrong and should not be told otherwise.
            let command = hick_lang::tag_text(tag);
            let absolute_use = command.contains(&format!("{path}/"))
                || command.contains(&format!("{path} "))
                || command.trim_end().ends_with(path);
            if !absolute_use {
                continue;
            }
            let rel = path.trim_start_matches('/');
            out.push(format!(
                "line {}: mounts at `{path}`, but mounts resolve UNDER the \
                 container workdir — `{path}` becomes `<workdir>{path}`. \
                 Commands must address it relatively (`{rel}/...`); an \
                 absolute `{path}` reaches the real filesystem root and fails.",
                tag.source_line
            ));
        }
    }
    out
}

/// A path this document has already ingested (a `hick:ingested` block's
/// child `hick:file` elements — the record of what a run actually produced)
/// that a SEPARATE, top-level `<hick:file path="…">` also declares.
///
/// `hick:file` content is written to host disk in one upfront pass before
/// any `hick:exec` cell runs, with no regard for document order. Once a path
/// is recorded inside `hick:ingested`, a second top-level declaration of the
/// exact same path either gets silently reverted (if the scaffolder cell
/// that produced it runs again with `--force`) or leaves the author with two
/// declarations disagreeing about what the file contains — the shape hit
/// authoring a tutorial that tried to give an ingested file an earlier,
/// smaller "stub" version via a second `hick:file`.
///
/// Deliberately an EXACT path match, not a directory/prefix match: an output
/// volume's prefix is an ordinary directory that legitimately holds many
/// hand-authored files alongside the ingested ones (this is the normal
/// shape, not a hazard — an earlier, prefix-based version of this check
/// warned on every one of them and was corrected after running it against a
/// real ingest-based document). A bare scaffolder cell that has not been
/// ingested yet cannot be checked this way either: its future output
/// filenames are not known until it runs, which is exactly the run this
/// check happens before.
///
/// A warning rather than a hard error:
/// the author may already know which declaration should win.
pub fn output_collision_warnings(doc: &hick_lang::HickDocument) -> Vec<String> {
    let ingested: HashMap<&str, usize> = doc
        .tags()
        .filter(|t| t.name == "exec")
        .flat_map(|t| t.child_tags())
        .filter(|t| t.name == "ingested")
        .flat_map(|t| t.child_tags())
        .filter(|t| t.name == "file")
        .filter_map(|t| t.get_attribute("path").map(|p| (p.trim(), t.source_line)))
        .collect();
    if ingested.is_empty() {
        return Vec::new();
    }

    doc.tags()
        .filter(|t| t.name == "file")
        .filter_map(|t| t.get_attribute("path").map(|p| (p.trim(), t.source_line)))
        .filter_map(|(path, line)| {
            let ingested_line = *ingested.get(path)?;
            Some(format!(
                "line {line}: `<hick:file path=\"{path}\">` declares a path \
                 already ingested at line {ingested_line}. `hick:file` content \
                 is written to disk before any cell runs, so this second \
                 declaration either gets silently reverted the next time the \
                 scaffolder cell runs with `--force`, or leaves the two \
                 declarations disagreeing about what the file contains. Drop \
                 this declaration and let the ingested content own the path, \
                 rather than hand-editing a competing copy."
            ))
        })
        .collect()
}

/// Containers declaring an image the local executor will not honour.
///
/// `LocalExecutor` records `image=` and ignores it: cells run against the
/// HOST toolchain. A document declaring `image="duckdb/duckdb:v1.5.5"` then
/// fails with `duckdb: not found` on a machine without duckdb — an error
/// that reads like a broken document rather than a missing local dependency,
/// and it is the first thing a new user hits after `cargo install`.
///
/// One line per document, listing the images, so the reader knows which
/// tools they are expected to have.
pub fn ignored_image_warning(doc: &hick_lang::HickDocument) -> Option<String> {
    let mut images: Vec<&str> = doc
        .tags()
        .filter(|t| t.name == "container")
        .filter_map(|t| t.get_attribute("image"))
        .filter(|i| *i != "host")
        .collect();
    images.sort_unstable();
    images.dedup();
    if images.is_empty() {
        return None;
    }
    Some(format!(
        "the local executor ignores image= — these cells run against your HOST \
         toolchain, so the tools in {} must be installed locally. Declared: {}",
        if images.len() == 1 {
            "this image"
        } else {
            "these images"
        },
        images.join(", ")
    ))
}
</book:entry>

</book:doc>
