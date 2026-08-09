import { useEffect, useState } from "react";

import { declaredSegment } from "../analytics/attribution";
import { emit } from "../analytics/events";
import { InterestSection } from "../components/InterestSection";
import { DECLARED_SEGMENTS, INTERESTS } from "../landing/interests";
import { navigate } from "../router";

// React StrictMode mounts every component twice in development. Without this
// the dev build reports two `landing_viewed` events per visit, which is the
// kind of instrumentation bug that is only noticed after it has poisoned a
// week of production funnel numbers.
let viewedFired = false;

/** Test seam: lets a test assert the once-per-load guarantee from a clean slate. */
export function resetViewedForTest() {
  viewedFired = false;
}

/**
 * The discovery landing page.
 *
 * One page, organized by the jobs and pains people arrive with — never by who
 * we think they are. See the header comment in src/landing/interests.ts for
 * why that distinction is the whole design, and
 * docs/specs/freeform/landing-discovery.md for what this page can and cannot
 * see about the people who visit it.
 */
export function LandingView() {
  const [declared, setDeclared] = useState<string | null>(() => declaredSegment());

  useEffect(() => {
    if (viewedFired) return;
    viewedFired = true;
    emit({ name: "landing_viewed", path: location.hash || "#/" });
  }, []);

  const cta = (ctaId: string, path: string) => () => {
    emit({ name: "cta_clicked", cta_id: ctaId });
    navigate(path);
  };

  const declare = (segment: string) => () => {
    emit({ name: "segment_declared", declared_segment: segment });
    setDeclared(segment);
  };

  return (
    <div className="landing">
      <header className="landing-hero">
        <h1>Documents that run, and fail loudly when they lie.</h1>
        <p className="landing-sub">
          A <code>.hick</code> file is prose, a program, a test suite, and an audit trail at
          once. Its examples execute on every commit; when the output drifts from what the
          document claims, the build goes red.
        </p>
        <div className="landing-cta">
          <button className="btn btn-primary" onClick={cta("hero-start", "/login")}>
            Start free
          </button>
          <button className="btn" onClick={cta("hero-pricing", "/pricing")}>
            See pricing
          </button>
        </div>
      </header>

      <div className="landing-interests">
        <p className="landing-interests-lead">
          Open whichever of these is actually your problem.
        </p>
        {INTERESTS.map((interest, index) => (
          <InterestSection key={interest.id} interest={interest} index={index} />
        ))}
      </div>

      <aside className="landing-anchor">
        {declared === null ? (
          <>
            <p className="landing-anchor-q">
              Optional, and it changes nothing on this page — what best describes you?
            </p>
            <div className="landing-anchor-options">
              {DECLARED_SEGMENTS.map((segment) => (
                <button key={segment.id} className="btn btn-quiet" onClick={declare(segment.id)}>
                  {segment.label}
                </button>
              ))}
            </div>
          </>
        ) : (
          <p className="landing-anchor-q muted">Thanks — that helps.</p>
        )}
      </aside>

      <footer className="landing-foot">
        <button className="btn btn-primary" onClick={cta("foot-start", "/login")}>
          Start free
        </button>
        <p className="muted">
          Open plan: unlimited public projects, 300 execution minutes a month, no card.
        </p>
      </footer>
    </div>
  );
}
