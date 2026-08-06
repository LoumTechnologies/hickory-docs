import { useEffect, useMemo, useRef, useState } from "react";
import type { TranscriptEvent } from "../api/types";
import { segmentsAt, transcriptDuration } from "../lib/transcript";

function formatTime(ms: number): string {
  const s = ms / 1000;
  return s < 10 ? s.toFixed(1) + "s" : Math.round(s) + "s";
}

/**
 * Animated terminal transcript playback: play/pause/scrub over timed events.
 * Plain <pre>-based rendering — no terminal emulator dependency.
 */
export function TranscriptPlayer({
  events,
  live = false,
}: {
  events: TranscriptEvent[];
  live?: boolean;
}) {
  const duration = useMemo(() => transcriptDuration(events), [events]);
  const [playhead, setPlayhead] = useState(duration);
  const [playing, setPlaying] = useState(false);
  const rafRef = useRef(0);
  const preRef = useRef<HTMLPreElement>(null);

  // While a run is live-streaming, pin the playhead to the end.
  useEffect(() => {
    if (live) {
      setPlaying(false);
      setPlayhead(duration);
    }
  }, [live, duration]);

  useEffect(() => {
    if (!playing) return;
    let last = performance.now();
    const tick = (now: number) => {
      const dt = now - last;
      last = now;
      setPlayhead((p) => {
        const next = p + dt;
        if (next >= duration) {
          setPlaying(false);
          return duration;
        }
        return next;
      });
      rafRef.current = requestAnimationFrame(tick);
    };
    rafRef.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafRef.current);
  }, [playing, duration]);

  useEffect(() => {
    const pre = preRef.current;
    if (pre) pre.scrollTop = pre.scrollHeight;
  }, [playhead, events]);

  const segments = useMemo(() => segmentsAt(events, playhead), [events, playhead]);

  const togglePlay = () => {
    if (playing) {
      setPlaying(false);
    } else {
      if (playhead >= duration) setPlayhead(0);
      setPlaying(true);
    }
  };

  return (
    <div className="transcript">
      <pre ref={preRef} className="terminal" data-testid="terminal">
        {segments.map((seg, i) =>
          seg.kind === "cmd" ? (
            <span key={i} className="t-cmd">
              <span className="t-prompt">$ </span>
              {seg.text}
              {"\n"}
            </span>
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
      {!live && duration > 0 && (
        <div className="transcript-controls">
          <button
            className="btn btn-icon"
            onClick={togglePlay}
            aria-label={playing ? "Pause" : "Play"}
          >
            {playing ? "⏸" : "▶"}
          </button>
          <input
            type="range"
            min={0}
            max={duration}
            step={16}
            value={playhead}
            aria-label="Scrub transcript"
            onChange={(e) => {
              setPlaying(false);
              setPlayhead(Number(e.target.value));
            }}
          />
          <span className="transcript-time">
            {formatTime(playhead)} / {formatTime(duration)}
          </span>
        </div>
      )}
    </div>
  );
}
