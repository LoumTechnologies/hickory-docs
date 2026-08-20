//! What a cell's captured output is *recorded* as.
//!
//! # The rule
//!
//! **Captured cell output is recorded with `\n` line endings on every
//! platform.** A `\r\n` pair in a cell's stdout or stderr becomes `\n` before
//! it reaches a transcript entry, a returned `String`, a woven document, or a
//! recording on disk.
//!
//! # Why it is here and not in the comparison
//!
//! A cell runs through `sh -c` on Unix and `cmd.exe /C` on Windows
//! (`LocalExecutor::shell`), and cmd's `echo` emits CRLF — there is no way to
//! ask it not to. `<hick:expect>` bodies, meanwhile, are text inside a `.hick`
//! file in git, so they are LF. Without this, a document that passes on macOS
//! reports drift on Windows for a line-ending reason rather than a real one,
//! and `match="exact"` is unusable there for any output containing a newline.
//! Measured on a real Windows machine with the shipped binary, not inferred:
//! the run reports `expectation FAILED: outputs differ only in trailing
//! whitespace or final newline` (issue #18).
//!
//! Normalising *here* rather than in `hick_literate::expect` keeps `exact`
//! meaning exact. The comparison is still byte-for-byte; what changed is that
//! the bytes it compares against are portable. The alternative — comparing
//! loosely — would make `exact` quietly mean "exact modulo whatever we decided
//! not to look at", and would leave the transcript, the woven `.md`, and the
//! `.hick-cache` recording still platform-specific, so a recording made on
//! Windows would still fail to serve a run on Linux.
//!
//! # Binary output
//!
//! Rewriting bytes is wrong for a cell that emits binary — and this path
//! cannot carry binary in the first place. [`Executor::execute`] returns
//! `String` and [`ExecTranscriptEntry::output`] is a `String`: captured output
//! is decoded with `String::from_utf8_lossy`, so any byte that is not valid
//! UTF-8 has already been replaced with U+FFFD before newline normalisation is
//! reached. Nothing that CRLF→LF would damage survives capture intact anyway.
//!
//! Bytes that must stay bytes have their own route and never pass through
//! here: a cell writes them to a file, and they leave the container through
//! [`Executor::extract_volume`], which is `Vec<u8>` end to end.
//!
//! The residual case is real and worth naming: valid UTF-8 text in which CRLF
//! is the *subject* — a cell asserting that some tool emits DOS line endings.
//! That claim can no longer be made about stdout directly; write the output to
//! a file and have the cell report the bytes (`od -c`, `xxd`), which is a
//! sturdier test of it regardless.
//!
//! # Lone `\r`
//!
//! Left alone, deliberately. cmd emits CRLF, so CRLF is the whole problem, and
//! a bare `\r` is load-bearing everywhere else it appears: a progress bar or a
//! spinner repaints one line with it, and rewriting those to `\n` would turn a
//! single line of woven output into hundreds. The narrowest rule that fixes
//! the reported problem is the one that stays.
//!
//! [`Executor::execute`]: crate::Executor::execute
//! [`Executor::extract_volume`]: crate::Executor::extract_volume
//! [`ExecTranscriptEntry::output`]: crate::ExecTranscriptEntry::output

use std::borrow::Cow;

/// Record captured text the way every platform records it: `\r\n` → `\n`.
///
/// Borrows when there is nothing to change, which is every capture on Unix.
pub fn normalize_captured_newlines(text: &str) -> Cow<'_, str> {
    if text.contains("\r\n") {
        Cow::Owned(text.replace("\r\n", "\n"))
    } else {
        Cow::Borrowed(text)
    }
}

/// [`normalize_captured_newlines`] for output that arrives in chunks.
///
/// `LocalExecutor` streams stdout and stderr in 8 KiB reads and timestamps
/// each chunk as its own transcript event, so a `\r\n` can straddle two of
/// them. Normalising each chunk on its own would miss exactly those pairs —
/// rarely, and only on outputs of the wrong length, which is the worst kind of
/// bug to have. This holds a trailing `\r` back until the next chunk says what
/// it was.
#[derive(Debug, Default)]
pub struct CapturedStream {
    pending_cr: bool,
}

impl CapturedStream {
    pub fn new() -> Self {
        Self::default()
    }

    /// Decode and normalise one chunk of captured bytes.
    pub fn push(&mut self, chunk: &[u8]) -> String {
        let mut bytes = Vec::with_capacity(chunk.len() + 1);
        if std::mem::take(&mut self.pending_cr) {
            bytes.push(b'\r');
        }
        bytes.extend_from_slice(chunk);
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
            self.pending_cr = true;
        }
        normalize_captured_newlines(&String::from_utf8_lossy(&bytes)).into_owned()
    }

    /// Emit anything held back once the stream has ended — a `\r` that turned
    /// out to be the last byte of the output rather than half of a `\r\n`.
    pub fn finish(&mut self) -> String {
        if std::mem::take(&mut self.pending_cr) {
            "\r".to_string()
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_becomes_lf() {
        assert_eq!(normalize_captured_newlines("one\r\ntwo\r\n"), "one\ntwo\n");
    }

    #[test]
    fn lf_only_output_is_untouched_and_not_copied() {
        assert!(matches!(
            normalize_captured_newlines("one\ntwo\n"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn a_lone_carriage_return_is_left_alone() {
        // A repainted progress line stays one line.
        assert_eq!(
            normalize_captured_newlines("10%\r50%\r100%\n"),
            "10%\r50%\r100%\n"
        );
    }

    #[test]
    fn a_crlf_split_across_chunks_is_still_normalised() {
        let mut s = CapturedStream::new();
        let first = s.push(b"one\r");
        let second = s.push(b"\ntwo\r\n");
        assert_eq!(format!("{first}{second}{}", s.finish()), "one\ntwo\n");
    }

    #[test]
    fn a_carriage_return_at_the_very_end_survives() {
        let mut s = CapturedStream::new();
        let first = s.push(b"done\r");
        assert_eq!(format!("{first}{}", s.finish()), "done\r");
    }

    #[test]
    fn an_empty_chunk_does_not_release_a_held_back_cr() {
        let mut s = CapturedStream::new();
        assert_eq!(s.push(b"x\r"), "x");
        assert_eq!(s.push(b""), "");
        assert_eq!(s.push(b"\ny"), "\ny");
        assert_eq!(s.finish(), "");
    }
}
