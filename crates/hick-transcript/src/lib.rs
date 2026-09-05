//! Transcript parsing, and the derivation of speaker turns from raw bytes.
//!
//! A transcript is bytes some other tool produced — a recorder, a meeting
//! assistant, a person typing fast. `<hick:transcript>` holds those bytes
//! verbatim (it is a raw-content element, so a meeting where somebody said
//! "the hick:copy tag" is not a parse error), and this crate derives the
//! speaker turns *from* them.
//!
//! Derived, not stored beside. The alternative — writing both the raw block
//! and a list of turns into the document — would create two representations of
//! one meeting that could disagree, and a new thing for `hick test` to check.
//! One source of truth with a projection over it cannot drift, because the
//! projection *is* the source read a second way.
//!
//! Every derived utterance carries a span into the original bytes, so lineage
//! can answer "Sam, at 00:14:03" rather than pointing at a wall of text.
//!
//! See `docs/specs/freeform/ingest.md`.

use hick_lang::{HickDocument, HickNode, HickTag, SourceSpan};

// ---------------------------------------------------------------------------
// Utterances
// ---------------------------------------------------------------------------

/// One speaker turn, located in the bytes it was read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utterance {
    /// Whoever the exporter said was talking. `None` when the format carries
    /// no attribution — which is information, not a defect.
    pub speaker: Option<String>,
    /// The timestamp as written in the source, never reformatted: it is a
    /// quotation, and normalising it would be the first small lie.
    pub at: Option<String>,
    /// The spoken text, with the speaker and timestamp furniture removed.
    pub text: String,
    /// Byte offset of `text` within the transcript source.
    pub start: usize,
    /// Byte offset past the end of `text`.
    pub end: usize,
}

/// The transcript shapes this can read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// WebVTT, as produced by most recorders and by `<track>` exports.
    WebVtt,
    /// SubRip.
    Srt,
    /// SubViewer, which is what Google Meet's caption export produces:
    /// `0:00:03.000,0:00:07.000` on its own line, then the payload. No `-->`
    /// and no cue index, so neither the WebVTT nor the SRT reader sees it.
    Sbv,
    /// Line-oriented exports: `Sam: …`, `**Sam** (00:14): …`,
    /// `[00:14] Sam: …`. Covers the markdown that meeting assistants emit and
    /// plain text somebody typed.
    Lines,
}

impl Format {
    /// The name this format answers to in a `format=` attribute.
    pub fn name(self) -> &'static str {
        match self {
            Format::WebVtt => "vtt",
            Format::Srt => "srt",
            Format::Sbv => "sbv",
            Format::Lines => "lines",
        }
    }

    /// Resolve a declared `format=` value, accepting the spellings people
    /// actually write.
    pub fn from_name(name: &str) -> Option<Format> {
        match name.trim().to_ascii_lowercase().as_str() {
            "vtt" | "webvtt" => Some(Format::WebVtt),
            "srt" | "subrip" => Some(Format::Srt),
            "sbv" | "subviewer" => Some(Format::Sbv),
            "lines" | "markdown" | "md" | "text" | "plain" | "txt" => Some(Format::Lines),
            _ => None,
        }
    }
}

/// Guess the format of `source`.
///
/// `None` means "not a shape this knows", which is a supported outcome: ingest
/// keeps the material and loses only the structure, rather than refusing a
/// file and leaving it in the inbox forever.
pub fn detect(source: &str) -> Option<Format> {
    let trimmed = source.trim_start_matches('\u{feff}').trim_start();
    if trimmed.starts_with("WEBVTT") {
        return Some(Format::WebVtt);
    }
    if looks_like_srt(trimmed) {
        return Some(Format::Srt);
    }
    if looks_like_sbv(trimmed) {
        return Some(Format::Sbv);
    }
    // Only claim `Lines` if something in it actually attributes a line to a
    // speaker. Prose with no attribution has no turns to derive, and saying it
    // does would invent structure that is not there.
    source
        .lines()
        .any(|line| split_speaker_line(line).is_some())
        .then_some(Format::Lines)
}

fn looks_like_srt(trimmed: &str) -> bool {
    let mut lines = trimmed.lines();
    let Some(first) = lines.next() else {
        return false;
    };
    if first.trim().is_empty() || !first.trim().chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    lines
        .next()
        .is_some_and(|second| second.contains("-->") && second.contains(','))
}

/// SubViewer opens straight into a `start,end` timing line.
fn looks_like_sbv(trimmed: &str) -> bool {
    trimmed
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| sbv_timing(line).is_some())
}

/// The start time of a `0:00:03.000,0:00:07.000` line.
///
/// Distinguished from SRT by having no `-->`: SRT spells the separator with an
/// arrow and uses commas as decimal points, so a line with a comma and no arrow
/// is SubViewer or is not a timing line at all.
fn sbv_timing(line: &str) -> Option<String> {
    let line = line.trim();
    if line.contains("-->") {
        return None;
    }
    let (start, end) = line.split_once(',')?;
    (is_clock(start) && is_clock(end)).then(|| start.trim().to_string())
}

fn is_clock(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.contains(':')
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || c == ':' || c == '.')
}

/// Read `source` as `format`, returning its speaker turns in order.
pub fn parse(source: &str, format: Format) -> Vec<Utterance> {
    match format {
        Format::WebVtt => parse_cues(source, true),
        Format::Srt | Format::Sbv => parse_cues(source, false),
        Format::Lines => parse_lines(source),
    }
}

/// Read `source`, guessing the format. Empty when nothing is recognisable.
pub fn parse_detected(source: &str) -> Vec<Utterance> {
    detect(source).map(|f| parse(source, f)).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Cue formats (WebVTT, SRT)
// ---------------------------------------------------------------------------

/// Both cue formats are the same shape: an optional identifier line, a timing
/// line containing `-->`, then payload lines until a blank one.
fn parse_cues(source: &str, vtt: bool) -> Vec<Utterance> {
    let mut out = Vec::new();
    let mut at: Option<String> = None;
    let mut payload: Vec<(usize, usize)> = Vec::new();
    let mut offset = 0usize;

    let flush =
        |at: &mut Option<String>, payload: &mut Vec<(usize, usize)>, out: &mut Vec<Utterance>| {
            if payload.is_empty() {
                *at = None;
                return;
            }
            let start = payload[0].0;
            let end = payload[payload.len() - 1].1;
            let joined: Vec<&str> = payload.iter().map(|(s, e)| &source[*s..*e]).collect();
            let raw = joined.join("\n");
            let (speaker, text_offset) = if vtt {
                voice_span(&raw)
            } else {
                match split_speaker_line(&raw) {
                    Some((who, _, rest)) => (Some(who), rest),
                    None => (None, 0),
                }
            };
            let payload_text = &raw[text_offset..];
            let lead = payload_text.len() - payload_text.trim_start().len();
            let text = payload_text.trim().to_string();
            if !text.is_empty() {
                out.push(Utterance {
                    speaker,
                    at: at.clone(),
                    text,
                    // A multi-line cue's span covers every line of it, which is
                    // what a reader means by "this turn". For such a turn the span
                    // is longer than the joined text (line endings differ), so
                    // lineage degrades to synthetic rather than mapping wrongly —
                    // which is the documented behaviour of a mismatched span.
                    start: start + text_offset + lead,
                    end,
                });
            }
            payload.clear();
            *at = None;
        };

    for line in source.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        let body = body.strip_suffix('\r').unwrap_or(body);
        let body_start = offset;
        let body_end = offset + body.len();
        offset += line.len();

        if body.trim().is_empty() {
            flush(&mut at, &mut payload, &mut out);
            continue;
        }
        if body.starts_with("WEBVTT") || body.starts_with("NOTE ") {
            continue;
        }
        if let Some(start_time) = timing_start(body).or_else(|| sbv_timing(body)) {
            // A timing line closes whatever came before it, in case the file
            // omits the blank separator.
            flush(&mut at, &mut payload, &mut out);
            at = Some(start_time);
            continue;
        }
        // A bare number before a timing line is a cue identifier, not speech.
        if payload.is_empty() && at.is_none() && body.trim().chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        payload.push((body_start, body_end));
    }
    flush(&mut at, &mut payload, &mut out);
    out
}

/// The start time of a `00:00:01.000 --> 00:00:04.000` line, as written.
fn timing_start(line: &str) -> Option<String> {
    let arrow = line.find("-->")?;
    let start = line[..arrow].trim();
    (!start.is_empty()
        && start
            .chars()
            .all(|c| c.is_ascii_digit() || c == ':' || c == '.' || c == ','))
    .then(|| start.to_string())
}

/// WebVTT voice spans: `<v Sam>text`, optionally closed with `</v>`.
///
/// Returns the speaker and the byte offset at which the spoken text begins.
fn voice_span(payload: &str) -> (Option<String>, usize) {
    if let Some(rest) = payload.strip_prefix("<v ")
        && let Some(close) = rest.find('>')
    {
        let speaker = rest[..close].trim();
        // `<v.loud Sam>` carries classes on the tag name; the name is last.
        let speaker = speaker.rsplit('.').next().unwrap_or(speaker).trim();
        return (
            (!speaker.is_empty()).then(|| speaker.to_string()),
            "<v ".len() + close + 1,
        );
    }
    match split_speaker_line(payload) {
        Some((who, _, rest)) => (Some(who), rest),
        None => (None, 0),
    }
}

// ---------------------------------------------------------------------------
// Line formats
// ---------------------------------------------------------------------------

fn parse_lines(source: &str) -> Vec<Utterance> {
    let mut out: Vec<Utterance> = Vec::new();
    let mut offset = 0usize;

    for line in source.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        let body = body.strip_suffix('\r').unwrap_or(body);
        let body_start = offset;
        offset += line.len();

        if body.trim().is_empty() {
            continue;
        }
        match split_speaker_line(body).filter(|(_, _, at)| !body[*at..].trim().is_empty()) {
            // A `Name:` with nothing after it is a heading, not a turn — which
            // is what `Summary:` on its own line is. Falling through to the
            // continuation arm keeps it as prose instead of inventing a
            // speaker who said nothing.
            Some((speaker, at, text_offset)) => {
                // The span has to cover the text and nothing else: trimming
                // here but not there would make lineage point one character to
                // the left of every line somebody said.
                let raw = &body[text_offset..];
                let lead = raw.len() - raw.trim_start().len();
                let text = raw.trim().to_string();
                let start = body_start + text_offset + lead;
                out.push(Utterance {
                    speaker: Some(speaker),
                    at,
                    end: start + text.len(),
                    text,
                    start,
                });
            }
            // A line with no attribution continues the turn above it. Wrapped
            // paragraphs are ordinary in these exports, and starting a new
            // unattributed turn for each one would shred the meeting.
            None => match out.last_mut() {
                Some(previous) => {
                    previous.text.push('\n');
                    previous.text.push_str(body.trim());
                    previous.end = body_start + body.len();
                }
                None => out.push(Utterance {
                    speaker: None,
                    at: None,
                    text: body.trim().to_string(),
                    start: body_start,
                    end: body_start + body.len(),
                }),
            },
        }
    }
    out
}

/// Split `Sam: …`, `**Sam** (00:14): …`, or `[00:14] Sam: …`.
///
/// Returns the speaker, any timestamp, and the byte offset where the spoken
/// text begins. Conservative on purpose: a false positive turns a sentence
/// containing a colon into a speaker named after its first four words, and a
/// transcript full of invented speakers is worse than one with none.
fn split_speaker_line(line: &str) -> Option<(String, Option<String>, usize)> {
    let mut cursor = 0usize;
    let mut at: Option<String> = None;

    // A leading `[00:14:03]` or `(00:14)` timestamp.
    let rest = &line[cursor..];
    let trimmed = rest.trim_start();
    cursor += rest.len() - trimmed.len();
    if let Some((close, open_len)) = trimmed
        .strip_prefix('[')
        .map(|r| (r.find(']'), 1))
        .or_else(|| trimmed.strip_prefix('(').map(|r| (r.find(')'), 1)))
        && let Some(close) = close
    {
        let inner = &trimmed[open_len..open_len + close];
        if is_timestamp(inner) {
            at = Some(inner.trim().to_string());
            cursor += open_len + close + 1;
        }
    }

    let rest = &line[cursor..];
    let trimmed = rest.trim_start();
    cursor += rest.len() - trimmed.len();

    // `**Sam**` — the markdown meeting assistants emit.
    let (name, after_name) = if let Some(inner) = trimmed.strip_prefix("**") {
        let close = inner.find("**")?;
        (&inner[..close], cursor + 2 + close + 2)
    } else {
        let colon = trimmed.find(':')?;
        (&trimmed[..colon], cursor + colon)
    };

    let name = name.trim();
    if !is_speaker_name(name) {
        return None;
    }

    // A trailing `(00:14)` between the name and the colon.
    let mut after = after_name;
    let rest = &line[after..];
    let trimmed = rest.trim_start();
    after += rest.len() - trimmed.len();
    if let Some(inner) = trimmed.strip_prefix('(')
        && let Some(close) = inner.find(')')
        && is_timestamp(&inner[..close])
    {
        at = Some(inner[..close].trim().to_string());
        after += close + 2;
    }

    let rest = &line[after..];
    let trimmed = rest.trim_start();
    after += rest.len() - trimmed.len();
    let text_start = after + trimmed.strip_prefix(':').map(|_| 1).unwrap_or(0);
    if text_start > after && trimmed.starts_with(':') {
        return Some((name.to_string(), at, text_start));
    }
    // `**Sam**` with no colon at all is still an attribution.
    if line[after_name..after].is_empty() && !trimmed.starts_with(':') {
        return None;
    }
    Some((name.to_string(), at, after))
}

fn is_timestamp(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.contains(':')
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || c == ':' || c == '.' || c == ',')
}

/// Is this plausibly a person's name rather than the first half of a sentence?
///
/// The failing case this exists for is `We discussed the following: indexes`,
/// which is prose with a colon in it and must not become a speaker named "We
/// discussed the following". The rule that separates them is capitalisation: a
/// multi-word name is Title Case, because that is how names are written and is
/// not how sentences are. A single word is allowed either way, so `sam` and
/// `@sam` work as handles.
fn is_speaker_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 60 {
        return false;
    }
    // Sentence punctuation means this was prose. `.` survives for `Sam H.`
    if name.contains([',', ';', '!', '?', '"', '(', ')', '[', ']']) {
        return false;
    }
    if !name
        .chars()
        .next()
        .is_some_and(|c| c.is_alphabetic() || c == '@' || c == '_')
    {
        return false;
    }
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.len() > 4 {
        return false;
    }
    if words.len() > 1 {
        return words.iter().all(|word| {
            word.chars()
                .next()
                .is_some_and(|c| c.is_uppercase() || c.is_numeric())
        });
    }
    true
}

// ---------------------------------------------------------------------------
// Derivation into the document
// ---------------------------------------------------------------------------

/// A speaker name reduced to something usable in a class.
pub fn slug(speaker: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true;
    for ch in speaker.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Replace every `<hick:transcript>`'s raw text with derived `<hick:said>`
/// turns, in place.
///
/// The document on disk is untouched: this is the projection, built for the
/// run that is about to happen. A transcript whose format is unrecognised
/// keeps its raw text and gains no turns — the material survives, only the
/// structure is missing.
pub fn expand(doc: &mut HickDocument) {
    expand_nodes(&mut doc.nodes);
}

fn expand_nodes(nodes: &mut [HickNode]) {
    for node in nodes.iter_mut() {
        if let HickNode::Tag(tag) = node {
            if tag.name == "transcript" {
                expand_transcript(tag);
            } else {
                expand_nodes(&mut tag.children);
            }
        }
    }
}

fn expand_transcript(tag: &mut HickTag) {
    let Some((raw, span)) = raw_child(tag) else {
        return;
    };
    let format = tag
        .get_attribute("format")
        .and_then(Format::from_name)
        .or_else(|| detect(&raw));
    let Some(format) = format else {
        return;
    };
    let utterances = parse(&raw, format);
    if utterances.is_empty() {
        return;
    }

    let base = tag.get_attribute("id").map(str::to_string);
    let line = tag.source_line;
    let column = tag.source_column;

    let mut children = Vec::with_capacity(utterances.len());
    for (index, utterance) in utterances.iter().enumerate() {
        let mut attributes = Vec::new();
        if let Some(base) = &base {
            attributes.push(("id".to_string(), format!("{base}-u{}", index + 1)));
        }
        let mut classes = vec!["said".to_string()];
        if let Some(speaker) = &utterance.speaker {
            let slug = slug(speaker);
            if !slug.is_empty() {
                classes.push(format!("said-{slug}"));
            }
            attributes.push(("by".to_string(), speaker.clone()));
        }
        attributes.push(("class".to_string(), classes.join(" ")));
        if let Some(at) = &utterance.at {
            attributes.push(("at".to_string(), at.clone()));
        }

        let text_span = span.map(|base| sub_span(base, &raw, utterance));
        children.push(HickNode::Tag(HickTag {
            name: "said".to_string(),
            attributes,
            children: vec![HickNode::Text(utterance.text.clone(), text_span)],
            self_closing: false,
            source_line: line,
            source_column: column,
            source_span: None,
            close_span: None,
        }));
    }
    tag.children = children;
}

/// The transcript's single raw text child, with its span.
fn raw_child(tag: &HickTag) -> Option<(String, Option<SourceSpan>)> {
    match tag.children.as_slice() {
        [HickNode::Text(text, span)] => Some((text.clone(), *span)),
        _ => None,
    }
}

/// Locate an utterance's text inside the span the raw block occupies.
///
/// Line and column are recomputed from the raw bytes so that lineage reports a
/// real position in the document rather than the top of the transcript.
fn sub_span(base: SourceSpan, raw: &str, utterance: &Utterance) -> SourceSpan {
    let start = utterance.start.min(raw.len());
    let end = utterance.end.min(raw.len()).max(start);
    let before = &raw[..start];
    let newlines = before.bytes().filter(|b| *b == b'\n').count();
    let col = match before.rfind('\n') {
        Some(nl) => before.len() - nl - 1,
        None => base.start_col + before.len(),
    };
    SourceSpan {
        start: base.start + start,
        end: base.start + end,
        start_line: base.start_line + newlines,
        start_col: col,
        file_id: base.file_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn speakers(utterances: &[Utterance]) -> Vec<Option<&str>> {
        utterances.iter().map(|u| u.speaker.as_deref()).collect()
    }

    fn texts(utterances: &[Utterance]) -> Vec<&str> {
        utterances.iter().map(|u| u.text.as_str()).collect()
    }

    // -----------------------------------------------------------------------
    // Cue formats
    // -----------------------------------------------------------------------

    const VTT: &str = "WEBVTT\n\n00:14:03.000 --> 00:14:11.000\n<v Sam>We can't ship until the index lands.\n\n00:14:19.000 --> 00:14:24.000\n<v Nate>Agreed — what's the write volume?\n";

    #[test]
    fn webvtt_voice_spans_name_the_speaker() {
        assert_eq!(detect(VTT), Some(Format::WebVtt));
        let out = parse(VTT, Format::WebVtt);
        assert_eq!(speakers(&out), vec![Some("Sam"), Some("Nate")]);
        assert_eq!(
            texts(&out),
            vec![
                "We can't ship until the index lands.",
                "Agreed — what's the write volume?"
            ]
        );
        assert_eq!(out[0].at.as_deref(), Some("00:14:03.000"));
    }

    #[test]
    fn a_timestamp_is_quoted_never_reformatted() {
        // It is somebody else's rendering of when a thing was said. Rewriting
        // it would be the first small lie in a document about provenance.
        let out = parse(VTT, Format::WebVtt);
        assert_eq!(out[1].at.as_deref(), Some("00:14:19.000"));
    }

    #[test]
    fn webvtt_falls_back_to_a_colon_attribution() {
        let src = "WEBVTT\n\n00:00:01.000 --> 00:00:04.000\nSam: the index is fine\n";
        assert_eq!(speakers(&parse(src, Format::WebVtt)), vec![Some("Sam")]);
    }

    #[test]
    fn webvtt_notes_and_cue_identifiers_are_not_speech() {
        let src = "WEBVTT\n\nNOTE recorded by a robot\n\n7\n00:00:01.000 --> 00:00:04.000\n<v Sam>hello\n";
        let out = parse(src, Format::WebVtt);
        assert_eq!(texts(&out), vec!["hello"]);
    }

    #[test]
    fn srt_is_read_as_cues() {
        let src = "1\n00:00:01,000 --> 00:00:04,000\nSam: the index is fine\n\n2\n00:00:05,000 --> 00:00:07,000\nNate: good\n";
        assert_eq!(detect(src), Some(Format::Srt));
        let out = parse(src, Format::Srt);
        assert_eq!(speakers(&out), vec![Some("Sam"), Some("Nate")]);
        assert_eq!(out[0].at.as_deref(), Some("00:00:01,000"));
    }

    #[test]
    fn google_meet_caption_exports_are_read() {
        // Meet exports captions as SubViewer: no `-->`, no cue index, so
        // neither the WebVTT nor the SRT reader recognises it.
        let src = "0:00:03.000,0:00:07.000\nSam: The migration finished overnight.\n\n0:00:08.000,0:00:12.000\nNate: Good, I'll close the ticket.\n";
        assert_eq!(detect(src), Some(Format::Sbv));
        let out = parse(src, Format::Sbv);
        assert_eq!(speakers(&out), vec![Some("Sam"), Some("Nate")]);
        assert_eq!(out[0].at.as_deref(), Some("0:00:03.000"));
        assert_eq!(texts(&out)[1], "Good, I'll close the ticket.");
    }

    #[test]
    fn srt_is_not_mistaken_for_subviewer() {
        // SRT uses commas as DECIMAL points, so "has a comma" alone cannot
        // tell them apart — the arrow is what does.
        let src = "1\n00:00:01,000 --> 00:00:04,000\nSam: hello\n";
        assert_eq!(detect(src), Some(Format::Srt));
    }

    #[test]
    fn a_multi_line_cue_stays_one_turn() {
        let src = "WEBVTT\n\n00:00:01.000 --> 00:00:04.000\n<v Sam>one\ntwo\n";
        let out = parse(src, Format::WebVtt);
        assert_eq!(texts(&out), vec!["one\ntwo"]);
    }

    // -----------------------------------------------------------------------
    // Line formats
    // -----------------------------------------------------------------------

    #[test]
    fn line_exports_are_read_in_their_common_spellings() {
        let src = "Sam: plain colon\n**Nate** (00:14): bold with a time\n[00:15:02] Ada: bracketed time\n";
        assert_eq!(detect(src), Some(Format::Lines));
        let out = parse(src, Format::Lines);
        assert_eq!(speakers(&out), vec![Some("Sam"), Some("Nate"), Some("Ada")]);
        assert_eq!(
            texts(&out),
            vec!["plain colon", "bold with a time", "bracketed time"]
        );
        assert_eq!(out[1].at.as_deref(), Some("00:14"));
        assert_eq!(out[2].at.as_deref(), Some("00:15:02"));
    }

    #[test]
    fn an_unattributed_line_continues_the_turn_above_it() {
        // Wrapped paragraphs are ordinary in these exports, and starting a new
        // turn per line would shred the meeting.
        let src = "Sam: first line\nsecond line of the same thought\nNate: reply\n";
        let out = parse(src, Format::Lines);
        assert_eq!(speakers(&out), vec![Some("Sam"), Some("Nate")]);
        assert_eq!(out[0].text, "first line\nsecond line of the same thought");
    }

    #[test]
    fn prose_with_a_colon_does_not_become_a_speaker() {
        // The false positive that matters: a transcript full of invented
        // speakers is worse than one with none.
        for line in [
            "We discussed the following: indexes, locks, and vacuum",
            "The plan is simple: ship it",
        ] {
            assert!(
                split_speaker_line(line).is_none(),
                "invented a speaker from: {line}"
            );
        }
    }

    #[test]
    fn a_heading_is_not_a_turn_by_a_speaker_who_said_nothing() {
        let src = "Summary:\nWe agreed to wait.\nSam: fine by me\n";
        let out = parse(src, Format::Lines);
        assert_eq!(speakers(&out), vec![None, Some("Sam")]);
        assert_eq!(out[0].text, "Summary:\nWe agreed to wait.");
    }

    #[test]
    fn handles_and_multiword_names_both_work() {
        assert!(is_speaker_name("sam"));
        assert!(is_speaker_name("@sam"));
        assert!(is_speaker_name("Sam Hendricks"));
        assert!(is_speaker_name("Sam H."));
        assert!(!is_speaker_name("we discussed the thing"));
    }

    #[test]
    fn prose_with_no_attribution_at_all_has_no_format() {
        // Claiming `Lines` here would invent structure that is not there.
        assert_eq!(detect("Just some notes.\nNo speakers anywhere.\n"), None);
        assert!(parse_detected("Just some notes.\n").is_empty());
    }

    // -----------------------------------------------------------------------
    // Spans
    // -----------------------------------------------------------------------

    #[test]
    fn an_utterance_locates_its_own_text_in_the_source() {
        // This is what lets lineage answer "Sam, at 00:14:03" instead of
        // pointing at the whole transcript.
        let src = "Sam: hello there\nNate: hi\n";
        let out = parse(src, Format::Lines);
        assert_eq!(&src[out[0].start..out[0].end], "hello there");
        assert_eq!(&src[out[1].start..out[1].end], "hi");
    }

    #[test]
    fn slugs_are_usable_in_a_class() {
        assert_eq!(slug("Sam Hendricks"), "sam-hendricks");
        assert_eq!(slug("@sam"), "sam");
        assert_eq!(slug("Ada  Lovelace!"), "ada-lovelace");
    }

    // -----------------------------------------------------------------------
    // Derivation
    // -----------------------------------------------------------------------

    fn expanded(body: &str) -> HickDocument {
        let mut doc = hick_lang::parse(body).expect("parses");
        expand(&mut doc);
        doc
    }

    /// `HickDocument::find_tags` is deliberately non-recursive, and derived
    /// turns live inside their transcript.
    fn find_deep<'a>(nodes: &'a [HickNode], name: &str, out: &mut Vec<&'a HickTag>) {
        for node in nodes {
            if let HickNode::Tag(tag) = node {
                if tag.name == name {
                    out.push(tag);
                }
                find_deep(&tag.children, name, out);
            }
        }
    }

    fn deep<'a>(doc: &'a HickDocument, name: &str) -> Vec<&'a HickTag> {
        let mut out = Vec::new();
        find_deep(&doc.nodes, name, &mut out);
        out
    }

    #[test]
    fn a_transcript_derives_addressable_turns() {
        let doc = expanded(
            "# Sync\n\n<hick:transcript id=\"t\">\nSam: the index is fine\nNate: good\n</hick:transcript>\n",
        );
        let said = deep(&doc, "said");
        assert_eq!(said.len(), 2);
        assert_eq!(said[0].get_attribute("id"), Some("t-u1"));
        assert_eq!(said[0].get_attribute("by"), Some("Sam"));
        assert_eq!(said[0].get_attribute("class"), Some("said said-sam"));
        assert_eq!(said[1].get_attribute("id"), Some("t-u2"));
    }

    #[test]
    fn an_unreadable_transcript_keeps_its_bytes_and_gains_no_turns() {
        // The material survives; only the structure is missing.
        let source = "# Sync\n\n<hick:transcript id=\"t\">\nJust prose, nobody attributed.\n</hick:transcript>\n";
        let doc = expanded(source);
        assert!(deep(&doc, "said").is_empty());
        let transcript = deep(&doc, "transcript")[0];
        assert!(transcript.text_content().contains("Just prose"));
    }

    #[test]
    fn a_transcript_may_contain_text_that_looks_like_markup() {
        // It is a raw-content element: a meeting where somebody said
        // "the hick:copy tag" is not a parse error.
        let doc = expanded(
            "<hick:transcript id=\"t\">\nSam: I used the <hick:copy id=\"x\"> tag\n</hick:transcript>\n",
        );
        let said = deep(&doc, "said");
        assert_eq!(said.len(), 1);
        assert!(said[0].text_content().contains("<hick:copy"));
        // The lookalike must not have become a real tag.
        assert!(deep(&doc, "copy").is_empty());
    }

    #[test]
    fn derived_turns_carry_spans_into_the_document() {
        let source = "<hick:transcript id=\"t\">\nSam: hello there\n</hick:transcript>\n";
        let doc = expanded(source);
        let said = deep(&doc, "said");
        let HickNode::Text(text, span) = &said[0].children[0] else {
            panic!("expected a text child");
        };
        let span = span.expect("derived turns carry spans");
        assert_eq!(&source[span.start..span.end], text.as_str());
    }

    #[test]
    fn a_transcript_without_an_id_still_derives_turns() {
        let doc = expanded("<hick:transcript>\nSam: hello\n</hick:transcript>\n");
        let said = deep(&doc, "said");
        assert_eq!(said.len(), 1);
        assert!(said[0].get_attribute("id").is_none());
        assert_eq!(said[0].get_attribute("class"), Some("said said-sam"));
    }

    #[test]
    fn a_transform_can_select_a_whole_transcript_or_one_speaker() {
        // The point of deriving turns: a summary can name what it came from.
        let doc = expanded(
            "<hick:transcript id=\"t\">\nSam: the index is fine\nNate: good\nSam: shipping\n</hick:transcript>\n",
        );

        let whole = hick_lang::fragments_matching(&doc, "#t");
        assert_eq!(whole.len(), 1, "a transcript is selectable as a fragment");
        assert_eq!(whole[0].name, "transcript");

        let one_turn = hick_lang::fragments_matching(&doc, "#t-u2");
        assert_eq!(one_turn.len(), 1);
        assert_eq!(one_turn[0].text_content(), "good");

        let by_speaker = hick_lang::fragments_matching(&doc, ".said-sam");
        assert_eq!(by_speaker.len(), 2, "both of Sam's turns");
        assert_eq!(by_speaker[1].text_content(), "shipping");
    }

    #[test]
    fn selecting_a_whole_transcript_does_not_also_select_its_turns() {
        // Otherwise a summary's input would contain every sentence twice.
        let doc =
            expanded("<hick:transcript id=\"t\" class=\"said\">\nSam: hello\n</hick:transcript>\n");
        let matched = hick_lang::fragments_matching(&doc, ".said");
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].name, "transcript");
    }

    #[test]
    fn a_declared_format_beats_detection() {
        let doc = expanded(
            "<hick:transcript id=\"t\" format=\"lines\">\nSam: hello\n</hick:transcript>\n",
        );
        assert_eq!(deep(&doc, "said").len(), 1);
    }
}
