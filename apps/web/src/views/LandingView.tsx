import { useEffect, useState } from "react";

import { declaredSegment } from "../analytics/attribution";
import { emit } from "../analytics/events";
import { AgentToolSurface } from "../components/AgentToolSurface";
import { InstallCommand } from "../components/InstallCommand";
import { InterestSection } from "../components/InterestSection";
import { ProgramDemo } from "../landing/demos/ProgramDemo";
import { DECLARED_SEGMENTS, INTERESTS } from "../landing/interests";

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
 *
 * The order of the two sections below the hero is deliberate and is the page's
 * whole argument. FIRST the surface your own agent drives, because that is the
 * reason to install this rather than a thing it also does. THEN one demo of
 * the mechanism underneath, because "your agent edits through hashes and
 * lineage" is a promise nobody should take on trust — the demo is where a
 * visitor checks it by driving the real weaver in their own browser.
 *
 * There is exactly one demo. The page used to carry three, two of which
 * showed things this product does not have (a hosted issue-tracker
 * integration, two people editing one document); see local-only.md. A demo of
 * an absent feature is not a smaller lie than a sentence claiming it.
 */
export function LandingView() {
  const [declared, setDeclared] = useState<string | null>(() => declaredSegment());

  useEffect(() => {
    if (viewedFired) return;
    viewedFired = true;
    emit({ name: "landing_viewed", path: location.hash || "#/" });
  }, []);

  const declare = (segment: string) => () => {
    emit({ name: "segment_declared", declared_segment: segment });
    setDeclared(segment);
  };

  return (
    <div className="landing">
      <header className="landing-hero">
        <h1>Give your coding agent a document it cannot lie in.</h1>
        <p className="landing-sub">
          Hickory Docs turns a repository into one your agent edits through content hashes and
          byte-exact lineage instead of guessing at line numbers — and it leaves its work behind as
          a document in git, with the commands it ran and what they actually printed. Bring Claude
          Code, Codex, or anything that speaks MCP.
        </p>
        {/* The call to action is what the product IS. hick is a program you
            install (docs/specs/freeform/local-only.md), so the primary action
            is installing it — there is no account to make and no server to
            sign in to. */}
        <div className="landing-cta">
          <InstallCommand />
        </div>
        <p className="landing-sub landing-install-note">
          Runs on your machine, on your files, in your repo, on your own API key. Free, no account,
          nothing to buy — and it never talks to us.
        </p>
      </header>

      <section className="landing-demo" aria-labelledby="agent-surface-h">
        <h2 id="agent-surface-h">Point your own agent at it</h2>
        <p className="landing-demo-lead">
          Three commands and five tools. Nothing to sign up for, no model of ours in the loop, and
          no harness you have to switch to.
        </p>
        <AgentToolSurface />
      </section>

      <section className="landing-demo" aria-labelledby="demo-program-h">
        <h2 id="demo-program-h">What your agent is editing</h2>
        <p className="landing-demo-lead">
          A document holds the code once. It weaves into the files that run, the files run, and an
          edit made at either end lands at the other. Drive it yourself — everything below happens
          in this page, with no account and no network.
        </p>
        <ProgramDemo />
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
