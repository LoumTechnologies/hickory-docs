import { useEffect, useRef, useState } from "react";

import { emit } from "../analytics/events";
import type { Interest } from "../landing/interests";

/**
 * One interest section: a heading titled by the job, a teaser, and a body
 * revealed on demand.
 *
 * The disclosure is the measurement. Opening a section is a deliberate act
 * that says "this is my problem", which is what makes the set of sections a
 * visitor opens a usable interest vector. Dwell is captured on close so a
 * headline that gets opened and immediately abandoned is distinguishable from
 * one that gets read — the first means the title promised the wrong thing.
 */
export function InterestSection({ interest, index }: { interest: Interest; index: number }) {
  const [open, setOpen] = useState(false);
  const openedAt = useRef<number | null>(null);

  const toggle = () => {
    if (open) {
      const dwell = openedAt.current === null ? 0 : Date.now() - openedAt.current;
      openedAt.current = null;
      emit({
        name: "interest_expanded",
        interest_id: interest.id,
        phase: "close",
        dwell_ms: dwell,
      });
    } else {
      openedAt.current = Date.now();
      emit({
        name: "interest_expanded",
        interest_id: interest.id,
        phase: "open",
        open_index: index,
      });
    }
    setOpen(!open);
  };

  // A visitor who reads a section and then closes the tab never triggers the
  // close handler, so their dwell would be lost — and "read it and left" is
  // not the same signal as "never opened it". `visibilitychange` is the only
  // unload-ish event mobile browsers fire reliably.
  useEffect(() => {
    if (!open) return;
    const flush = () => {
      if (document.visibilityState !== "hidden" || openedAt.current === null) return;
      emit({
        name: "interest_expanded",
        interest_id: interest.id,
        phase: "close",
        dwell_ms: Date.now() - openedAt.current,
      });
      openedAt.current = null;
    };
    document.addEventListener("visibilitychange", flush);
    return () => document.removeEventListener("visibilitychange", flush);
  }, [open, interest.id]);

  const bodyId = `interest-body-${interest.id}`;
  return (
    <section className={`interest${open ? " open" : ""}`}>
      <h3>
        <button aria-expanded={open} aria-controls={bodyId} onClick={toggle}>
          <span className="interest-caret" aria-hidden="true">
            {open ? "▾" : "▸"}
          </span>
          <span className="interest-title">{interest.title}</span>
        </button>
      </h3>
      <p className="interest-teaser">{interest.teaser}</p>
      <div id={bodyId} className="interest-body" hidden={!open}>
        {interest.body.map((paragraph) => (
          <p key={paragraph.slice(0, 32)}>{paragraph}</p>
        ))}
        {interest.sample && (
          <pre className="interest-sample">
            <code>{interest.sample}</code>
          </pre>
        )}
        {interest.links && (
          <p className="interest-links">
            {interest.links.map((link) => (
              <a
                key={link.id}
                href={link.href}
                onClick={() =>
                  emit({
                    name: "interest_clicked",
                    interest_id: interest.id,
                    link_id: link.id,
                  })
                }
              >
                {link.label}
              </a>
            ))}
          </p>
        )}
      </div>
    </section>
  );
}
