import { useEffect, useMemo, useRef } from "react";
import type { TranscriptEvent } from "../api/types";
import { segmentsAt, transcriptDuration } from "../lib/transcript";

/**
 * A cell's terminal output, shown whole.
 *
 * No play button, no scrubber, no replay: a run's output is a result to
 * read, not a recording to perform. (Timed events still exist in the
 * transcript data — they order interleaved stdout/stderr correctly — but
 * the only motion here is a live run appending as it happens.)
 */
export function Transcript({
  events,
  live = false,
}: {
  events: TranscriptEvent[];
  live?: boolean;
}) {
  // In a cell the command is the source text directly above the panel, so
  // echoing it again as `$ cmd` is pure repetition — hide cmd events unless
  // there are several (a multi-command group whose outputs interleave, where
  // the prompts are needed to tell which output belongs to which command).
  const showCommands = useMemo(
    () => events.filter((e) => e.kind === "cmd").length > 1,
    [events],
  );
  const segments = useMemo(
    () => segmentsAt(events, transcriptDuration(events)),
    [events],
  );
  const preRef = useRef<HTMLPreElement>(null);

  // A live run appends; keep the newest output in view.
  useEffect(() => {
    const pre = preRef.current;
    if (live && pre) pre.scrollTop = pre.scrollHeight;
  }, [segments, live]);

  return (
    <div className="transcript">
      <pre ref={preRef} className="terminal" data-testid="terminal">
        {segments.map((seg, i) =>
          seg.kind === "cmd" ? (
            showCommands ? (
              <span key={i} className="t-cmd">
                <span className="t-prompt">$ </span>
                {seg.text}
                {"\n"}
              </span>
            ) : null
          ) : seg.kind === "exit" ? (
            seg.exitCode === 0 ? null : (
              <span key={i} className="t-exit">
                [{seg.text}]{"\n"}
              </span>
            )
          ) : (
            <span key={i} className={seg.kind === "err" ? "t-err" : "t-out"}>
              {seg.text}
            </span>
          ),
        )}
        {live && <span className="t-cursor">▋</span>}
      </pre>
    </div>
  );
}
