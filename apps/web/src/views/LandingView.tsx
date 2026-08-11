import { useEffect, useState } from "react";

import { declaredSegment } from "../analytics/attribution";
import { emit } from "../analytics/events";
import { InstallCommand } from "../components/InstallCommand";
import { InterestSection } from "../components/InterestSection";
import { CollaborationDemo } from "../landing/demos/CollaborationDemo";
import { KnowledgeWorkDemo } from "../landing/demos/KnowledgeWorkDemo";
import { ProgramDemo } from "../landing/demos/ProgramDemo";
import { DECLARED_SEGMENTS, INTERESTS } from "../landing/interests";
import { config } from "../config";
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
        <h1>An AI agent whose work you can trace, byte by byte.</h1>
        <p className="landing-sub">
          Hickory Docs sits where an ordinary agent session, literate programming, and the
          semantic web meet: the agent works inside a document, every artifact it produces is
          woven from a named piece of that document, and the path between the two stays
          walkable in both directions.
        </p>
        {/* The call to action is what the product IS. hickory is a program you
            install (docs/specs/freeform/local-first.md), so the primary action
            is installing it — not signing up for a workspace that no longer
            exists. The hosted app keeps its own front door while it runs. */}
        <div className="landing-cta">
          <InstallCommand />
          <div className="landing-cta-secondary">
            <button className="btn" onClick={cta("hero-pricing", "/pricing")}>
              See pricing
            </button>
            {config.hosted && (
              <button className="btn" onClick={cta("hero-start", "/login")}>
                Or use the hosted workspace
              </button>
            )}
          </div>
        </div>
        <p className="landing-sub landing-install-note">
          Runs on your machine, on your files, in your repo. Share a live
          session with <code>hickory serve --share</code> — no account, for
          you or for them.
        </p>
      </header>

      <section className="landing-demo" aria-labelledby="demo-knowledge-h">
        <h2 id="demo-knowledge-h">Here is how it works</h2>
        <p className="landing-demo-lead">
          Walk the six steps. Everything below runs in this page — no account, no sign-up.
        </p>
        <KnowledgeWorkDemo />
      </section>

      <section className="landing-demo" aria-labelledby="demo-program-h">
        <h2 id="demo-program-h">The point is information flow, in any kind of work</h2>
        <p className="landing-demo-lead">
          Tracking where a claim came from is not a software problem, and Hickory Docs is not a
          software tool that happens to do prose. It is the other way round — which is why the
          same document can also <em>be</em> the program it describes.
        </p>
        <ProgramDemo />
      </section>

      <section className="landing-demo" aria-labelledby="demo-collab-h">
        <h2 id="demo-collab-h">Two people, one document, stored in your own repository</h2>
        <p className="landing-demo-lead">
          Live collaboration on top of git. Notes, documentation and source code live in the
          same place, with the same history, on GitHub.
        </p>
        <CollaborationDemo />
      </section>

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
        <InstallCommand />
        <p className="muted">
          Free, and yours: documents are files in your own repository, execution
          happens on your own hardware, and nothing needs an account.
        </p>
      </footer>
    </div>
  );
}
