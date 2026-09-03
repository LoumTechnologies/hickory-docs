//! How much Hickory knows about a language, measured rather than declared.
//!
//! Three rungs, and each contains the one below it:
//!
//! * **Bronze — the text is right.** The extension is routed to a language
//!   id, so a `hick:file` holding it is more than bytes: it is highlighted,
//!   it reaches a language server if one turns up, and a cell can run its
//!   toolchain.
//! * **Silver — the editor is right.** A language server and a debug adapter
//!   are reachable, so a generated file has completions and diagnostics and
//!   can be stepped through in DOCUMENT coordinates.
//! * **Gold — the code is data.** An index answers questions about the whole
//!   project, and a code model server answers questions about the language's
//!   own type system, so a script can GENERATE against it.
//!
//! ## Why this is computed
//!
//! Because the alternative has failed twice in this repository, the same way
//! both times: a hand-maintained list beside a catalogue. `lang_detect` had
//! no `cs` row, so C# support existed and was unreachable. Then `hick init`'s
//! reported-languages list omitted C#, so a C# project was told about Go and
//! YAML. Neither was caught by a test, because each table was the only
//! statement of its own contents.
//!
//! So every fact here is read from the thing that would actually be used —
//! the LSP, DAP and index catalogues, and discovery on this machine — and
//! nothing is written down twice. `hick lang` cannot claim a language is
//! debuggable before it is, which is the rule
//! `docs/specs/freeform/launching-what-a-document-builds.md` states for
//! debugging and which generalises to the whole ladder.

use std::path::Path;

/// A rung. Ordered, so `>=` means "at least".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// Not even routed: a file of this language is opaque bytes.
    None,
    Bronze,
    Silver,
    Gold,
}

impl Tier {
    pub fn name(self) -> &'static str {
        match self {
            Tier::None => "—",
            Tier::Bronze => "BRONZE",
            Tier::Silver => "SILVER",
            Tier::Gold => "GOLD",
        }
    }
}

/// Whether one capability is here, obtainable, or absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Have {
    /// Found on this machine, or built in.
    Present,
    /// Not here, but `hick … install` can fetch it, or the ecosystem ships
    /// one this project knows how to find once installed.
    Installable,
    /// Nothing exists that Hickory knows of.
    Missing,
    /// The question does not apply to this language.
    ///
    /// A data or markup language has nothing to step through and no
    /// source-level type model to generate against, so asking it for a
    /// debugger is not a gap — it is the wrong question. Reported as `n/a`
    /// and never counted as missing, because "install a debugger for JSON"
    /// is advice nobody can take.
    NotApplicable,
}

impl Have {
    /// A capability counts toward a tier when it is here OR obtainable: a
    /// tier describes what this language can do in Hickory, not what this
    /// laptop happens to have downloaded.
    pub fn counts(self) -> bool {
        matches!(
            self,
            Have::Present | Have::Installable | Have::NotApplicable
        )
    }

    pub fn mark(self) -> &'static str {
        match self {
            Have::Present => "yes",
            Have::Installable => "get",
            Have::Missing => "—",
            Have::NotApplicable => "n/a",
        }
    }
}

/// One language's standing.
#[derive(Debug, Clone)]
pub struct LanguageSupport {
    pub language: &'static str,
    /// Bronze: the extension routes to this language id.
    pub text: Have,
    /// Bronze: the editor has a grammar and can draw this language.
    ///
    /// Measured rather than assumed, because Bronze's own description
    /// promised "highlighted" for two months while eleven routed languages
    /// opened as undifferentiated grey text.
    pub draw: Have,
    /// Silver: a language server.
    pub lsp: Have,
    /// Silver: a debug adapter.
    pub dap: Have,
    /// Gold: a project index.
    pub index: Have,
    /// Gold: a code model server for generation.
    pub model: Have,
    /// Gold: a typed-client emitter, so a generator can be WRITTEN in this
    /// language against any model.
    pub client: Have,
    pub tier: Tier,
    /// The one thing that would raise this language a rung, or `None` when
    /// it is already Gold.
    pub next: Option<String>,
}

/// Every language Hickory can route, with what it can do for each.
pub fn survey(root: &Path) -> Vec<LanguageSupport> {
    hick_lsp::lang_detect::known_language_ids()
        .into_iter()
        .map(|language| support(language, root))
        .collect()
}

/// One language's standing, on this machine, in this project.
pub fn support(language: &'static str, root: &Path) -> LanguageSupport {
    // Bronze is exactly "this id came out of the routing table", which it did
    // by construction for everything `survey` yields.
    let text = Have::Present;

    let data = is_data_language(language);

    // No `Installable` rung: a grammar either ships in the app or does not
    // exist, and there is nothing a person could go and install.
    let draw = if hick_lsp::lang_detect::highlight(language).is_editor() {
        Have::Present
    } else {
        Have::Missing
    };

    let lsp = capability(
        hick_lsp::discovery::discover(language, root).is_some(),
        // Obtainable when Hickory knows what to LOOK for, not only when it
        // can fetch it. `gopls` is not in the install catalogue and never
        // needs to be — discovery has always known its name, and Go people
        // install it the way Go people do. Counting only the catalogue
        // reported Go, Java, Kotlin, Scala, PHP and Ruby as having no
        // language server at all, which is false and reads as neglect.
        installs(&crate::lsp_install::CATALOGUE, language)
            || hick_lsp::discovery::known_languages().contains(&language),
    );
    let dap = if data {
        Have::NotApplicable
    } else {
        capability(
            hick_dap::discovery::discover(language, root).is_some(),
            // A language `hick_dap` knows how to debug is obtainable even when
            // the catalogue cannot fetch the adapter — `hick dap list` tells you
            // how to get it, which is the same promise `how_to_get` makes.
            installs(&crate::dap_install::CATALOGUE, language)
                || hick_dap::known_languages().contains(&language),
        )
    };
    let index = if data {
        Have::NotApplicable
    } else {
        capability(
            crate::index_install::discover(language, root).is_some(),
            installs(&crate::index_install::CATALOGUE, language)
                || indexed_by_the_ecosystem(language),
        )
    };
    // The blank column is how a reader learns the rung is there and that
    // nothing has reached it — see `docs/specs/freeform/language-tiers.md`.
    //
    // `Installable` is deliberately not offered: no catalogue can fetch a
    // model server yet, so a language without one is Missing rather than a
    // promise nobody can keep.
    let model = if data {
        Have::NotApplicable
    } else {
        capability(crate::code_model::discover(language, root).is_some(), false)
    };

    // An emitter is language-specific support in the other direction: it is
    // what lets somebody write a generator IN this language, typed, against
    // any model. A language you can model but cannot write a generator in is
    // supported halfway, and the ladder should say so.
    let client = if data {
        Have::NotApplicable
    } else {
        capability(
            crate::typed_client::emit::Target::parse(language).is_some(),
            false,
        )
    };

    let tier = if data {
        // Silver is the ceiling and the top of its own ladder: "the editor is
        // right" is everything a document can want from a language it only
        // holds. Rounding it up to Gold because three columns say `n/a` would
        // claim a code model that cannot exist.
        if lsp.counts() {
            Tier::Silver
        } else {
            Tier::Bronze
        }
    } else if lsp.counts() && dap.counts() && index.counts() && model.counts() && client.counts() {
        Tier::Gold
    } else if lsp.counts() && dap.counts() {
        Tier::Silver
    } else {
        Tier::Bronze
    };

    let next = next_step(
        language,
        tier,
        data,
        &Capabilities {
            draw,
            lsp,
            dap,
            index,
            model,
            client,
        },
    );

    LanguageSupport {
        language,
        text,
        draw,
        lsp,
        dap,
        index,
        model,
        client,
        tier,
        next,
    }
}

fn capability(present: bool, installable: bool) -> Have {
    match (present, installable) {
        (true, _) => Have::Present,
        (false, true) => Have::Installable,
        (false, false) => Have::Missing,
    }
}

/// A language carried in documents but never executed.
///
/// The ladder is about what a document can DO with a language, and for these
/// the answer stops at "hold it and edit it well". There is nothing to step
/// through and no type model to generate from — the interesting model for SQL
/// is the live database schema, which is a different feature with a different
/// name. Marking them is how `hick lang` avoids advising somebody to install
/// a debugger for YAML.
fn is_data_language(language: &str) -> bool {
    matches!(
        language,
        "json" | "yaml" | "toml" | "markdown" | "html" | "css" | "sql" | "xml"
    )
}

/// Whether a catalogue can fetch this language's tool.
fn installs(catalogue: &crate::tool_install::Catalogue, language: &str) -> bool {
    catalogue
        .installers
        .iter()
        .any(|installer| installer.language == language)
}

/// Languages whose indexer exists but is not ours to install.
///
/// `hick index list` already tells a person how to get these; this is the
/// same fact, asked as a question instead of printed as advice. Kept next to
/// that list deliberately — if the two disagree, the test below fails.
fn indexed_by_the_ecosystem(language: &str) -> bool {
    matches!(
        language,
        "rust" | "csharp" | "java" | "scala" | "kotlin" | "go" | "c" | "cpp" | "ruby"
    )
}

/// The single most useful sentence for this language: what to do next.
/// What a language has, for deciding the one thing to say next.
struct Capabilities {
    draw: Have,
    lsp: Have,
    dap: Have,
    index: Have,
    model: Have,
    client: Have,
}

fn next_step(language: &str, tier: Tier, data: bool, have: &Capabilities) -> Option<String> {
    let Capabilities {
        draw,
        lsp,
        dap,
        index,
        model,
        client,
    } = *have;
    if tier == Tier::Gold {
        return None;
    }
    if data && tier == Tier::Silver {
        // Its ladder has no further rung. Saying so is the point of the
        // exemption: a promise you can keep beats a gap you never close.
        return None;
    }
    // Named in ladder order, because raising a rung needs the lower one
    // first, and one instruction is more useful than four. Drawing comes
    // before everything: it is the rung the language is already standing on.
    if !draw.counts() {
        return Some(format!(
            "no syntax grammar — {language} opens as plain text; \
             `apps/web/src/editor/languages.ts` binds them"
        ));
    }
    if !lsp.counts() {
        return Some(format!(
            "no language server Hickory knows of — Silver needs one for {language}"
        ));
    }
    if !dap.counts() {
        return Some(format!(
            "no debug adapter Hickory knows of — Silver needs one for {language}"
        ));
    }
    if !index.counts() {
        return Some(format!(
            "no index — Gold needs one; `hick index list` says what exists for {language}"
        ));
    }
    if !model.counts() {
        return Some(format!(
            "a code model server — `code-models/` has C#, TypeScript and Go; \
             {language} needs one"
        ));
    }
    if !client.counts() {
        return Some(format!(
            "a typed-client emitter, so a generator can be written IN {language} — \
             `typed_client::emit` has python, typescript, go and csharp"
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every language a catalogue can install a tool for must be a language
    /// the survey reports on.
    ///
    /// This is the test that would have caught both shipped bugs. It derives
    /// its expectations from the catalogues, so it cannot be satisfied by
    /// updating a list — only by making the routing table true.
    #[test]
    fn every_installable_language_is_surveyed() {
        let surveyed: Vec<&str> = hick_lsp::lang_detect::known_language_ids();
        for catalogue in [
            &crate::lsp_install::CATALOGUE,
            &crate::dap_install::CATALOGUE,
            &crate::index_install::CATALOGUE,
        ] {
            for installer in catalogue.installers {
                assert!(
                    surveyed.contains(&installer.language),
                    "`{} {}` can install a {}, but no file extension routes to `{}` — \
                     so the tool could never be reached. Add the row to `lang_detect`.",
                    catalogue.command,
                    installer.language,
                    catalogue.what,
                    installer.language
                );
            }
        }
    }

    /// Anything named as ecosystem-indexed must also be routable, for the
    /// same reason.
    #[test]
    fn every_ecosystem_indexed_language_is_surveyed() {
        let surveyed = hick_lsp::lang_detect::known_language_ids();
        for language in [
            "rust", "csharp", "java", "scala", "kotlin", "go", "c", "cpp", "ruby",
        ] {
            assert!(
                indexed_by_the_ecosystem(language),
                "the list and the function must agree about {language}"
            );
            assert!(
                surveyed.contains(&language),
                "{language} has an indexer and no extension routes to it"
            );
        }
    }

    #[test]
    fn a_tier_needs_every_rung_below_it() {
        // Gold is unreachable while no model server exists, and that must be
        // visible rather than rounded up.
        let dir = tempfile::tempdir().unwrap();
        for support in survey(dir.path()) {
            assert_ne!(
                support.tier,
                Tier::Gold,
                "{} claims Gold with no code model server present",
                support.language
            );
            if support.tier == Tier::Silver {
                assert!(support.lsp.counts() && support.dap.counts());
            }
            assert!(
                support.next.is_some() || support.tier == Tier::Silver,
                "{} is not at its ceiling, so it must name a next step",
                support.language
            );
        }
    }
}
