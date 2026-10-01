import { useEffect, useState } from "react";

import { declaredSegment } from "../analytics/attribution";
import { emit } from "../analytics/events";
import { AgentToolSurface } from "../components/AgentToolSurface";
import { Convergence } from "../components/Convergence";
import { InstallCommand } from "../components/InstallCommand";
import { InterestSection } from "../components/InterestSection";
import { TooltipLayer } from "../components/TooltipLayer";
import { ProgramDemo } from "../landing/demos/ProgramDemo";
import { IntelligenceDemo } from "../landing/demos/IntelligenceDemo";
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
 * The order of the three sections below the hero is deliberate and is the
 * page's whole argument — why, then how, then proof:
 *
 * 1. `Convergence` — the claim. Session, reasoning and edit are one artifact.
 * 2. `AgentToolSurface` — the mechanism that delivers it: an MCP server your
 *    existing agent already speaks.
 * 3. `ProgramDemo` — the document itself, driven in the visitor's browser,
 *    because none of the above should be taken on trust.
 *
 * What this ordering exists to prevent: an earlier version led with
 * content-hash edit anchors as though they were the innovation. They are not
 * — every serious harness has solved edit application, and pitching a solved
 * problem as a breakthrough loses exactly the reader who knows the field. The
 * anchors are now an aside inside section 2, where they belong: load-bearing,
 * unremarkable.
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
        <h1>Literate programming + AI agents</h1>
        <p className="landing-sub">
          What do you get when you cross Claude Code, literate programming, and namespaced markup?
        </p>
        <p className="landing-sub">
          You get Hickory Docs: a local command-line program and MCP server for writing documents
          that read like an agent session and behave like a program. One document holds the prose,
          the code, and the runs — and weaves out the source files and the documentation, the way
          literate programming always did.
        </p>
        <p className="landing-sub">
          Install it, then run <code>hick init</code> in the repo you work in. That registers the
          MCP server for the agent you already use, so it can write <code>.md</code> documents and
          edit their generated files — an edit at either end lands at the other, byte-exactly.
          Editors get <code>hick-lsp</code>, which ships in the same download: it spawns the real
          language servers for each generated file and maps their diagnostics back onto the
          document, so embedded code gets the same tooling the generated code gets.{" "}
          <code>hick init</code> points it at whichever servers this repo already uses.
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
          nothing to buy.
        </p>
      </header>

      <section className="landing-demo" aria-labelledby="converge-h">
        <h2 id="converge-h">Three things that should have been one</h2>
        <p className="landing-demo-lead">
          Literate programming had this right in 1984 and could not make it stick. The generated
          files were the ones everyone actually edited, so the document, the source, and the
          documentation drifted apart — and writing inside the document meant giving up the
          completion, navigation, and diagnostics you had when editing the source directly. Both of
          those are fixed here: an edit to a generated file maps back into the document, and the
          language servers follow the code into it.
        </p>
        <Convergence />
      </section>

      <section className="landing-demo" aria-labelledby="agent-surface-h">
        <h2 id="agent-surface-h">How it reaches your agent</h2>
        <p className="landing-demo-lead">
          One command and five tools, over MCP. Nothing to sign up for, no model of ours in the
          loop, and no harness you have to switch to.
        </p>
        <AgentToolSurface />
      </section>

      <section className="landing-demo" aria-labelledby="demo-program-h">
        <h2 id="demo-program-h">What the document actually is</h2>
        <p className="landing-demo-lead">
          It holds the code once, weaves it into the files that run, and carries an edit made at
          either end back to the other — which is why the record cannot drift from the thing it
          describes. Its last cell pins a claim about <em>shape</em> rather than output: SCIP
          indexes the woven files and counts the ways the dependency could point backwards, and the
          document fails the day that count stops being zero. It is a file in your repository, not
          a session that evaporates. Drive it yourself; everything below happens in this page, with
          no account and no network.
        </p>
        <ProgramDemo />
      </section>

      <section className="landing-demo" aria-labelledby="demo-intel-h">
        <h2 id="demo-intel-h">Your editor still works inside the document</h2>
        <p className="landing-demo-lead">
          The usual objection to literate programming is that you give up your tools: the code
          becomes prose, and prose has no go-to-definition. It does not here. A document's code
          blocks are woven into virtual files, handed to the language servers already on your
          machine, and every answer is mapped back to the line you are looking at — so hover,
          completion, diagnostics and colouring work on the document itself. The editor below is
          the app's, and the compiler answering it is real.
        </p>
        <IntelligenceDemo />
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
      <TooltipLayer />
    </div>
  );
}
