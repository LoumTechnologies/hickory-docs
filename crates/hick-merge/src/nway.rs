//! The merged view's N-way alignment: which regions N sources agree on.
//!
//! See `docs/specs/freeform/the-merged-view.md`. One tab shows a file as it
//! exists in several places at once — two worktrees on different branches, a
//! checkout on another machine — unified into a single surface. Regions all
//! sources agree on appear **once**; regions that differ appear as
//! **variants**, in the shape `<hick:when>` already has.
//!
//! **None of that text exists on disk.** There is no file containing those
//! conditionals, nothing to commit, and nothing to merge. It is a lens, not a
//! document — the same category as a diff view.
//!
//! ## The sources are PEERS
//!
//! Decided 2026-08-23, and it is the decision the edit routing waits on. The
//! N sources are symmetric: **shared means agreed by all of them**, and there
//! is no order, because the view removes the question. The alternative — a
//! stack, where each branch lands on the one below — makes "shared" ambiguous
//! (agreed by everything, or inherited from the branch below?) and answering
//! that late would mean rewriting the routing.
//!
//! ## Deliberately conservative
//!
//! Aligning N sources so shared regions genuinely correspond — rather than
//! fusing two regions that merely look alike — is the main technical risk in
//! the whole idea, and **a bad alignment routes an edit silently into the
//! wrong file.** The failure of over-sharing is an edit in the wrong place;
//! the failure of under-sharing is a little redundant typing. So this errs the
//! second way, on purpose:
//!
//! - A line is shared only when **every** source has it, aligned, in the same
//!   place relative to its neighbours.
//! - Alignment is anchored on the first source and computed pairwise against
//!   it. A line that a diff had to guess about is not shared.
//! - Nothing is fused on similarity. Two lines are the same line or they are
//!   not.

use std::collections::BTreeMap;

/// One source in a merged view: where its bytes came from, and what they are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// How a person names this source — a worktree name, a branch, a machine.
    /// Shown on a variant, and what an edit is routed to.
    pub name: String,
    pub text: String,
}

/// A run of lines every source agrees on, or a place where they differ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Region {
    /// Every source has these lines, here. Editing them writes to all of
    /// them, which is what makes them **agreed by construction**: they cannot
    /// conflict later, because they never diverged.
    Shared { lines: Vec<String> },
    /// The sources differ here. One entry per source, in the order the
    /// sources were given, including the empty ones — a source that has
    /// nothing here is a fact about it, not an absence.
    Variant {
        /// Source name → its lines in this region.
        by_source: BTreeMap<String, Vec<String>>,
    },
}

impl Region {
    pub fn is_shared(&self) -> bool {
        matches!(self, Region::Shared { .. })
    }

    /// The lines this region contributes to the rendered view, for a shared
    /// region; empty for a variant, which renders per source.
    pub fn shared_lines(&self) -> &[String] {
        match self {
            Region::Shared { lines } => lines,
            Region::Variant { .. } => &[],
        }
    }
}

/// The synthesized view: the regions, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergedView {
    pub sources: Vec<String>,
    pub regions: Vec<Region>,
}

impl MergedView {
    /// How much of the view every source agrees on, as a fraction of shared
    /// lines over all lines rendered. Reported rather than computed by the
    /// caller, because the honest reading of a low number is "this view is
    /// mostly variants" and that is worth saying at the top of the tab.
    pub fn shared_line_count(&self) -> usize {
        self.regions.iter().map(|r| r.shared_lines().len()).sum()
    }

    pub fn variant_count(&self) -> usize {
        self.regions.iter().filter(|r| !r.is_shared()).count()
    }
}

/// Split `text` into lines, keeping their terminators, so a region can be
/// written back byte-for-byte. A file with no trailing newline keeps not
/// having one.
fn lines_with_endings(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, ch) in text.char_indices() {
        if ch == '\n' {
            out.push(text[start..=i].to_string());
            start = i + ch.len_utf8();
        }
    }
    if start < text.len() {
        out.push(text[start..].to_string());
    }
    out
}

/// Which lines of `spine` survive unchanged into `other`, as a set of spine
/// indices, using a line-level longest-common-subsequence.
///
/// `similar`'s diff is what the reverse-edit path already uses, so the notion
/// of "the same line" is one notion across the product.
fn aligned_with_spine(spine: &[String], other: &[String]) -> Vec<Option<usize>> {
    use similar::{Algorithm, capture_diff_slices};
    let ops = capture_diff_slices(Algorithm::Myers, spine, other);
    let mut map = vec![None; spine.len()];
    for op in ops {
        if let similar::DiffOp::Equal {
            old_index,
            new_index,
            len,
        } = op
        {
            for k in 0..len {
                map[old_index + k] = Some(new_index + k);
            }
        }
    }
    map
}

/// Synthesize the merged view of `sources`.
///
/// The first source is the spine; every other source is aligned against it.
/// A spine line is SHARED only when every other source has it, aligned — and
/// additionally only when the alignment is monotonic across all sources, so a
/// line that moved in one source relative to another is a variant rather than
/// a silently reordered "shared" line.
///
/// One source is trivially all-shared. Zero sources is an empty view rather
/// than an error: a view over nothing is a legible thing to open by mistake.
pub fn merged_view(sources: &[Source]) -> MergedView {
    let names: Vec<String> = sources.iter().map(|s| s.name.clone()).collect();
    if sources.is_empty() {
        return MergedView {
            sources: names,
            regions: Vec::new(),
        };
    }

    let per_source: Vec<Vec<String>> = sources
        .iter()
        .map(|s| lines_with_endings(&s.text))
        .collect();
    let spine = &per_source[0];

    // For every spine line, where it sits in each other source (or nowhere).
    let maps: Vec<Vec<Option<usize>>> = per_source[1..]
        .iter()
        .map(|other| aligned_with_spine(spine, other))
        .collect();

    // A spine line is a candidate when every source has it.
    let mut shared: Vec<bool> = (0..spine.len())
        .map(|i| maps.iter().all(|m| m[i].is_some()))
        .collect();

    // Monotonicity: a shared line must come after the previous shared line in
    // EVERY source. A line that appears in all sources but in a different
    // order is a coincidence, not a correspondence — fusing it would route an
    // edit into the wrong place, which is the failure this must not have.
    let mut last: Vec<Option<usize>> = vec![None; maps.len()];
    for i in 0..spine.len() {
        if !shared[i] {
            continue;
        }
        let ok = maps.iter().enumerate().all(|(s, m)| match (last[s], m[i]) {
            (None, Some(_)) => true,
            (Some(prev), Some(cur)) => cur > prev,
            _ => false,
        });
        if ok {
            for (s, m) in maps.iter().enumerate() {
                last[s] = m[i];
            }
        } else {
            shared[i] = false;
        }
    }

    // Walk the spine, emitting shared runs and the variant gaps between them.
    // A gap's content per source is everything between the previous shared
    // line and the next one, in that source's own coordinates.
    let mut regions = Vec::new();
    let mut cursor: Vec<usize> = vec![0; sources.len()];
    let mut i = 0usize;
    let mut pending_gap_start: Vec<usize> = cursor.clone();

    /// Emit the variant region covering `from..to` in each source's own
    /// coordinates. A source with nothing here still gets an entry: having
    /// nothing is a fact about it, not an absence.
    fn flush_gap(
        regions: &mut Vec<Region>,
        names: &[String],
        per_source: &[Vec<String>],
        from: &[usize],
        to: &[usize],
    ) {
        let mut by_source = BTreeMap::new();
        let mut any = false;
        for (s, lines) in per_source.iter().enumerate() {
            let start = from[s].min(lines.len());
            let end = to[s].min(lines.len()).max(start);
            let slice: Vec<String> = lines[start..end].to_vec();
            if !slice.is_empty() {
                any = true;
            }
            by_source.insert(names[s].clone(), slice);
        }
        if any {
            regions.push(Region::Variant { by_source });
        }
    }

    while i < spine.len() {
        if !shared[i] {
            i += 1;
            continue;
        }
        // Where this shared line sits in every source.
        let mut at = vec![i];
        for m in &maps {
            at.push(m[i].expect("a shared line is present in every source"));
        }
        // Everything since the last shared line is a variant gap.
        if at.iter().zip(pending_gap_start.iter()).any(|(a, b)| a > b) {
            flush_gap(&mut regions, &names, &per_source, &pending_gap_start, &at);
        }
        // Collect the maximal run of shared lines starting here.
        let mut run = Vec::new();
        let mut j = i;
        while j < spine.len() && shared[j] {
            // The run must stay contiguous in every source too, or it is two
            // runs with a gap between them.
            if j > i {
                let contiguous = maps
                    .iter()
                    .enumerate()
                    .all(|(s, m)| m[j] == Some(at[s + 1] + (j - i)));
                if !contiguous {
                    break;
                }
            }
            run.push(spine[j].clone());
            j += 1;
        }
        regions.push(Region::Shared { lines: run.clone() });
        for (s, c) in cursor.iter_mut().enumerate() {
            *c = if s == 0 { j } else { at[s] + run.len() };
        }
        pending_gap_start = cursor.clone();
        i = j;
    }

    // The tail after the last shared line.
    let ends: Vec<usize> = per_source.iter().map(|l| l.len()).collect();
    if ends
        .iter()
        .zip(pending_gap_start.iter())
        .any(|(a, b)| a > b)
    {
        flush_gap(&mut regions, &names, &per_source, &pending_gap_start, &ends);
    }

    MergedView {
        sources: names,
        regions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(name: &str, text: &str) -> Source {
        Source {
            name: name.to_string(),
            text: text.to_string(),
        }
    }

    fn shared_text(view: &MergedView) -> String {
        view.regions
            .iter()
            .flat_map(|r| r.shared_lines().iter().cloned())
            .collect()
    }

    #[test]
    fn identical_sources_are_entirely_shared() {
        let view = merged_view(&[src("a", "one\ntwo\n"), src("b", "one\ntwo\n")]);
        assert_eq!(view.variant_count(), 0);
        assert_eq!(shared_text(&view), "one\ntwo\n");
    }

    #[test]
    fn one_source_is_trivially_all_shared() {
        let view = merged_view(&[src("a", "one\ntwo\n")]);
        assert_eq!(view.variant_count(), 0);
        assert_eq!(shared_text(&view), "one\ntwo\n");
    }

    #[test]
    fn a_view_over_nothing_is_empty_rather_than_an_error() {
        let view = merged_view(&[]);
        assert!(view.regions.is_empty());
    }

    #[test]
    fn a_line_only_one_source_has_is_a_variant() {
        let view = merged_view(&[src("a", "one\nMINE\ntwo\n"), src("b", "one\ntwo\n")]);
        assert_eq!(shared_text(&view), "one\ntwo\n");
        assert_eq!(view.variant_count(), 1);
        let Region::Variant { by_source } = &view.regions[1] else {
            panic!("expected a variant: {:?}", view.regions);
        };
        assert_eq!(by_source["a"], vec!["MINE\n".to_string()]);
        // A source that has nothing here is a FACT about it, not an absence.
        assert!(by_source["b"].is_empty());
    }

    #[test]
    fn shared_means_agreed_by_all_and_not_by_most() {
        // Peers, not a stack: two out of three is not agreement.
        let view = merged_view(&[
            src("a", "one\nSAME\ntwo\n"),
            src("b", "one\nSAME\ntwo\n"),
            src("c", "one\nDIFFERENT\ntwo\n"),
        ]);
        assert_eq!(shared_text(&view), "one\ntwo\n");
        assert!(!shared_text(&view).contains("SAME"));
    }

    #[test]
    fn a_line_that_merely_looks_alike_elsewhere_is_not_fused() {
        // The failure this alignment must not have: over-sharing routes an
        // edit into the wrong file. Reordering is a coincidence, not a
        // correspondence.
        let view = merged_view(&[src("a", "x\ny\n"), src("b", "y\nx\n")]);
        // At most one of the two can be shared, never both.
        assert!(view.shared_line_count() <= 1, "{:?}", view.regions);
    }

    #[test]
    fn a_file_with_no_trailing_newline_keeps_not_having_one() {
        let view = merged_view(&[src("a", "one\ntwo"), src("b", "one\ntwo")]);
        assert_eq!(shared_text(&view), "one\ntwo");
    }

    #[test]
    fn every_source_is_reconstructible_from_the_view() {
        // The invariant that makes routing possible at all: walking the
        // regions in order, taking each source's own side of every variant,
        // gives that source back byte-for-byte.
        let sources = vec![
            src("a", "head\nalpha\nmiddle\nA-only\ntail\n"),
            src("b", "head\nbeta\nmiddle\ntail\n"),
            src("c", "head\ngamma\nmiddle\ntail\nextra\n"),
        ];
        let view = merged_view(&sources);
        for source in &sources {
            let mut rebuilt = String::new();
            for region in &view.regions {
                match region {
                    Region::Shared { lines } => rebuilt.extend(lines.iter().cloned()),
                    Region::Variant { by_source } => {
                        rebuilt.extend(by_source[&source.name].iter().cloned())
                    }
                }
            }
            assert_eq!(
                rebuilt, source.text,
                "source {} did not round-trip",
                source.name
            );
        }
    }

    #[test]
    fn a_completely_disjoint_pair_shares_nothing_and_still_round_trips() {
        let sources = vec![src("a", "aaa\nbbb\n"), src("b", "ccc\nddd\n")];
        let view = merged_view(&sources);
        assert_eq!(view.shared_line_count(), 0);
        for source in &sources {
            let mut rebuilt = String::new();
            for region in &view.regions {
                if let Region::Variant { by_source } = region {
                    rebuilt.extend(by_source[&source.name].iter().cloned());
                }
            }
            assert_eq!(rebuilt, source.text);
        }
    }
}

// ---------------------------------------------------------------------------
// Writing back through the view
// ---------------------------------------------------------------------------

/// Where an edit made in the view is routed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// One source only — the "just here" gesture. In the old design this
    /// wrapped text into a conditional in a file; here it simply means *write
    /// this to one target*, which is both easier to implement and easier to
    /// explain.
    JustHere { source: String },
    /// Every source. A shared edit writes N branches in one keystroke, which
    /// is a very sharp tool: it is what makes shared regions agreed by
    /// construction, and it is why undo across targets has to be designed
    /// rather than assumed.
    Shared,
}

/// What one target should end up containing after an edit through the view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Write {
    pub source: String,
    pub text: String,
}

/// Rebuild one source's full text from the view.
///
/// This is the round-trip invariant the whole feature rests on: walking the
/// regions in order and taking each source's own side of every variant gives
/// that source back byte-for-byte.
pub fn rebuild(view: &MergedView, source: &str) -> Option<String> {
    if !view.sources.iter().any(|s| s == source) {
        return None;
    }
    let mut out = String::new();
    for region in &view.regions {
        match region {
            Region::Shared { lines } => out.extend(lines.iter().cloned()),
            Region::Variant { by_source } => {
                out.extend(by_source.get(source)?.iter().cloned());
            }
        }
    }
    Some(out)
}

/// The writes an edit to region `index` produces.
///
/// `replacement` is the region's new text. For a shared region under
/// [`Route::Shared`] every source is rewritten; under
/// [`Route::JustHere`] only the named one is, and the region stops being
/// shared for everyone else — which is exactly the divergence the person
/// asked for.
///
/// Returns an error rather than a partial answer when the region index or the
/// source name does not exist: a write routed at nothing must not look like a
/// write that landed.
pub fn writes_for_edit(
    view: &MergedView,
    index: usize,
    replacement: &str,
    route: &Route,
) -> Result<Vec<Write>, String> {
    let region = view
        .regions
        .get(index)
        .ok_or_else(|| format!("no region {index} in this view"))?;
    if let Route::JustHere { source } = route
        && !view.sources.iter().any(|s| s == source)
    {
        return Err(format!(
            "no source named {source:?} in this view — it has {}",
            view.sources.join(", ")
        ));
    }

    let targets: Vec<String> = match route {
        Route::JustHere { source } => vec![source.clone()],
        Route::Shared => view.sources.clone(),
    };

    let mut out = Vec::new();
    for target in targets {
        let mut text = String::new();
        for (i, r) in view.regions.iter().enumerate() {
            let piece: String = if i == index {
                // The edited region, for this target. A variant edited "just
                // here" replaces only that source's side.
                replacement.to_string()
            } else {
                match r {
                    Region::Shared { lines } => lines.concat(),
                    Region::Variant { by_source } => by_source
                        .get(&target)
                        .map(|l| l.concat())
                        .unwrap_or_default(),
                }
            };
            text.push_str(&piece);
        }
        let _ = region;
        out.push(Write {
            source: target,
            text,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod write_tests {
    use super::*;

    fn view() -> MergedView {
        merged_view(&[
            Source {
                name: "a".into(),
                text: "head\nA\ntail\n".into(),
            },
            Source {
                name: "b".into(),
                text: "head\nB\ntail\n".into(),
            },
        ])
    }

    #[test]
    fn rebuilding_gives_each_source_back() {
        let v = view();
        assert_eq!(rebuild(&v, "a").unwrap(), "head\nA\ntail\n");
        assert_eq!(rebuild(&v, "b").unwrap(), "head\nB\ntail\n");
        assert!(rebuild(&v, "nope").is_none());
    }

    #[test]
    fn a_shared_edit_writes_every_source() {
        // A shared edit writes N branches in one keystroke. That is what
        // makes a shared region agreed by construction — and why it is a
        // sharp tool.
        let v = view();
        let shared = v
            .regions
            .iter()
            .position(|r| r.is_shared())
            .expect("a shared region");
        let writes = writes_for_edit(&v, shared, "HEAD\n", &Route::Shared).unwrap();
        assert_eq!(writes.len(), 2);
        assert_eq!(writes[0].text, "HEAD\nA\ntail\n");
        assert_eq!(writes[1].text, "HEAD\nB\ntail\n");
    }

    #[test]
    fn just_here_writes_one_source_and_leaves_the_others_alone() {
        // The gesture survives and gets simpler: it no longer wraps text into
        // a conditional in a file, it means "write this to one target".
        let v = view();
        let shared = v.regions.iter().position(|r| r.is_shared()).unwrap();
        let writes = writes_for_edit(
            &v,
            shared,
            "HEAD\n",
            &Route::JustHere { source: "a".into() },
        )
        .unwrap();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].source, "a");
        assert_eq!(writes[0].text, "HEAD\nA\ntail\n");
    }

    #[test]
    fn editing_a_variant_touches_only_that_side() {
        let v = view();
        let variant = v.regions.iter().position(|r| !r.is_shared()).unwrap();
        let writes =
            writes_for_edit(&v, variant, "AA\n", &Route::JustHere { source: "a".into() }).unwrap();
        assert_eq!(writes[0].text, "head\nAA\ntail\n");
    }

    #[test]
    fn a_write_routed_at_nothing_is_an_error_not_a_no_op() {
        // A write that landed nowhere must not look like one that landed.
        let v = view();
        assert!(writes_for_edit(&v, 99, "x", &Route::Shared).is_err());
        assert!(
            writes_for_edit(
                &v,
                0,
                "x",
                &Route::JustHere {
                    source: "ghost".into()
                }
            )
            .is_err()
        );
    }
}
