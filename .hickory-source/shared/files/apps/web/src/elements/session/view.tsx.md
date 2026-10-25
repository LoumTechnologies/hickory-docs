<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/session/view.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Session File Is The Conversation, And Opens As One](../../../../../../../guarantees/agent/a-session-is-the-conversation/contract.md)
- [An Answer In The Agent Pane Has Ribbons](../../../../../../../guarantees/agent/an-answer-in-the-agent-pane-has-ribbons/contract.md)
- [Captured reading surfaces share owned native actions](../../../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md)

Captured SHA-256: `c48f7f5be7b053ebd0597cac092d51b13b791c40cb3254461dd8edeb3d87e32c`.

<book:entry path="apps/web/src/elements/session/view.tsx">
import { useId, useMemo, type ReactNode } from "react";
import { useUiCommands, useUiRevision } from "../../lib/uiActionBridge";
import { useUiReading } from "../../lib/uiReading";
import { EvidenceDisclosure } from "../../components/EvidenceDisclosure";
import { openReview, reviewError } from "../../lib/reviews";
import { publishReading, openReading } from "../../lib/readingViews";
import { AcpRecord } from "../../components/AcpRecord";
// The conversation's elements, drawn as the cards the agent pane has always
// drawn — now in place of their source in the session document, with line
// numbers beside them. See docs/guarantees/agent/a-session-is-the-conversation.md
// and docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.
//
// What a reader sees first is the conversation: your words, the agent's
// answer, and under each answer the files it rests on. Everything the
// agent did on the way — its reasoning, the tools it called and what they
// returned — is there, folded, one click away; and the record's own
// bookkeeping (usage, the protocol marker) is drawn as nothing at all.
import type { Block, SessionLink } from "../../api/types";
import { matchBlock } from "../../lib/blockMatch";
import { openLocation } from "../../lib/revealLine";
import type { RenderedSlot } from "../../editor/rendered";
import type { ElementView, SlotContext, SlotKind } from "../types";

type Of<K extends Block["kind"]> = Extract<Block, { kind: K }>;

/** The server's block for this slot, when the session view has arrived. */
function serverBlock<K extends Block["kind"]>(
  slot: RenderedSlot,
  cx: SlotContext,
  kind: K,
): Of<K> | undefined {
  const same = (cx.sessionBlocks ?? []).filter((b): b is Of<K> => b.kind === kind);
  return matchBlock({ span: slot.span, index: slot.index }, same);
}

function view(kind: SlotKind, name: string, render: ElementView["render"]): ElementView {
  return { kind, draws: (block) => block.name === name, render };
}

/** Open a file the conversation named, at its first line when it has
 * lines — the same road a definition or a stack frame takes. */
function openTarget(link: SessionLink) {
  if (link.historical?.content !== undefined) {
    const snapshot = link.historical;
    const id = `evidence:${snapshot.sha256}:${snapshot.path}:${snapshot.first_line}:${link.span[0]}`;
    const quoted = snapshot.quoted_literals?.map(quote => `${JSON.stringify(quote.literal)}: ${quote.present_in_recorded_passage ? "occurs in recorded passage" : "absent from recorded passage"}`).join("\n") ?? "";
    const content = `# Historical evidence\n\nSource: ${snapshot.path}, lines ${snapshot.first_line}–${snapshot.last_line}\n\nSnapshot: ${snapshot.sha256}\n\nCurrent source: ${snapshot.current_state ?? "unchecked"}\n\n${snapshot.claim ? `## Citing paragraph\n\n${snapshot.claim}\n\n` : ""}${quoted ? `## Literal quote comparison\n\n${quoted}\n\nLiteral occurrence does not establish attribution or semantic support.\n\n` : ""}## Returned source passage\n\n${snapshot.content}`;
    publishReading(id, { title: "Historical evidence", path: "evidence.md", before: content, after: content, status: "historical · semantic support not assessed" });
    openReading(id, "Historical evidence");
    return;
  }
  openLocation(link.to.path, link.to.lines?.[0] ?? 1);
}

function reviewSourceChip(link: SessionLink) {
  const snapshot = link.historical;
  if (!snapshot?.content || !snapshot.path || !snapshot.sha256 || snapshot.first_line === undefined || snapshot.last_line === undefined || !/^[a-f0-9]{64}$/i.test(snapshot.sha256)) {
    reviewError("Open the source and select a passage to capture a review target; this chip has no exact historical source snapshot.");
    return false;
  }
  openReview({ kind: "source-relationship", path: snapshot.path, quote: snapshot.content,
    source_hash: snapshot.sha256, lines: [snapshot.first_line, snapshot.last_line],
    relationship: { family: link.family, claim: snapshot.claim, target: link.to,
      historical: snapshot, source_span: link.span } });
  return true;
}

const VERB: Record<SessionLink["family"], string> = {
  context: "read",
  lineage: "wrote",
  declared: "cites",
};

/** The files an answer rests on, as chips: what the turn read (context),
 * wrote (lineage) and pointed at (declared) — the same three families the
 * ribbons draw, in words, under the answer they belong to. */
export function SourceChips({ links }: { links: readonly SessionLink[] }) {
  const seen = new Set<string>();
  const shown = links.filter((link) => {
    const key = `${link.family}:${link.to.path}:${link.to.lines?.join("-") ?? ""}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  const parent = `session-sources:${useId()}`;
  const capture = useMemo(() => crypto.randomUUID(), [links]);
  const revision = useUiRevision(() => capture);
  useUiReading("conversation source chips", shown.map(link => ({ family: link.family, title: link.title, target: link.to, evidence_path: link.evidence?.path ?? null,
    opens: link.historical?.content !== undefined ? "captured historical passage" : "current source location" })));
  useUiCommands(shown.flatMap((link, index) => [
    { key: `${parent}:${index}:open`, command: { label: `Open recorded source ${link.to.path}`, parent, subject: parent, parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async () => { openTarget(link); return { status: "completed" as const, value: { navigation_requested: true, source_verified: false } }; } } },
    { key: `${parent}:${index}:review`, command: { label: `Challenge recorded source ${link.to.path}`, parent, subject: parent, parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async () => reviewSourceChip(link)
        ? { status: "completed" as const, value: { review_requested: true, review_completed: false } }
        : { status: "requires_input" as const, reason: "This chip has no exact historical source snapshot. Open the source and select a passage to capture a review target." } } },
  ]));
  if (!shown.length) return null;
  return (
    <div className="chat-sources" aria-label="Recorded inputs and declared references">
      {shown.map((link, i) => (
        <button
          key={i}
          type="button"
          className={`chat-source chat-source--${link.family}`}
          data-tip={`${link.title} · right-click or Alt+Enter to challenge this source relationship`}
          onContextMenu={event => { event.preventDefault(); reviewSourceChip(link); }}
          onKeyDown={event => { if (event.altKey && event.key === "Enter") { event.preventDefault(); reviewSourceChip(link); } }}
          onClick={() => openTarget(link)}
        >
          <span className="chat-source__verb">{link.title.startsWith("ACP-reported") ? "reports" : link.evidence ? "produces" : VERB[link.family]}</span>{" "}
          {link.evidence && <span className="mono">{link.evidence.path} → </span>}
          <span className="mono">{link.to.path.split("/").pop() ?? link.to.path}</span>
          {link.to.lines ? (
            <span className="chat-source__lines">
              {" "}
              {link.to.lines[0] === link.to.lines[1]
                ? link.to.lines[0]
                : `${link.to.lines[0]}–${link.to.lines[1]}`}
            </span>
          ) : null}
        </button>
      ))}
    </div>
  );
}

function SessionReading({ label, value, children }: { label: string; value: unknown; children: ReactNode }) {
  useUiReading(label, value);
  return <>{children}</>;
}

export const sessionUserView = view("session-user", "user", (slot, cx) => (
  <div className="chat-msg chat-user">
    <span className="chat-role">you</span>
    <div className="chat-bubble">
      <SessionReading label="recorded user message" value={serverBlock(slot, cx, "session-user")?.body ?? slot.text.trim()}><p className="chat-answer">{serverBlock(slot, cx, "session-user")?.body ?? slot.text.trim()}</p></SessionReading>
    </div>
  </div>
));

/** The blocks nested inside this one — an answer's reasoning and tool
 * calls — in order, with a slot each so their own views draw them. */
function nested(slot: RenderedSlot, cx: SlotContext): { block: Block; slot: RenderedSlot }[] {
  // The slot's own span, in the editor's units; the lens hands the server's
  // blocks over in the same units.
  const [from, to] = slot.span;
  return (cx.sessionBlocks ?? [])
    .filter((b) => b.kind !== "session-assistant" && b.span[0] > from && b.span[1] <= to)
    .map((block) => ({
      block,
      slot: { ...slot, span: block.span, at: block.span[0], text: "", key: `${slot.key}:${block.span[0]}` },
    }));
}

export const sessionAssistantView = view("session-assistant", "assistant", (slot, cx) => {
  // The prose is the answer; the reasoning and the tool calls nested in it
  // are its work, drawn inside the card, folded.
  const body =
    serverBlock(slot, cx, "session-assistant")?.body ?? slot.text.replace(/<hick:[\s\S]*$/, "").trim();
  const links = cx.sessionLinksAt?.get(slot.at) ?? [];
  const work = nested(slot, cx).filter(({ block }) => block.kind !== "session-meta");
  // A step with nothing to show — the agent only emitted a protocol marker
  // — is not an answer, and "no answer" under it reads as failure.
  if (!body && links.length === 0 && work.length === 0) {
    return <div className="chat-msg chat-agent chat-agent--silent" />;
  }
  // The work — reasoning, tool calls — is not speech: it sits above the
  // bubble as plain folds with no tail. The bubble, with its tail, holds
  // only what the agent said and what it rests on.
  return (
    <div className="chat-msg chat-agent">
      {work.length > 0 && (
        <div className="chat-work">
          {work.map(({ block, slot: inner }) => {
            const drawer = INNER[block.kind];
            return drawer ? <div key={inner.key} data-session-from={inner.span[0]} data-session-to={inner.span[1]}>{drawer(inner, cx)}</div> : null;
          })}
        </div>
      )}
      {(body || links.length > 0) && (
        <>
          <span className="chat-role">agent</span>
          <div className="chat-bubble">
            {body ? <SessionReading label="recorded assistant message" value={body}><p className="chat-answer">{body}</p></SessionReading> : null}
            <SourceChips links={links} />
          </div>
        </>
      )}
    </div>
  );
});

/** Which views draw an answer's nested work, by kind — filled in below,
 * once those views exist. */
const INNER: Partial<Record<Block["kind"], ElementView["render"]>> = {};

export const sessionReasoningView = view("session-reasoning", "reasoning", (slot, cx) => (
  <EvidenceDisclosure className="chat-reasoning" label="reasoning" value={serverBlock(slot, cx, "session-reasoning")?.body ?? slot.text.trim()}>
    <pre className="chat-reasoning__text">{serverBlock(slot, cx, "session-reasoning")?.body ?? slot.text.trim()}</pre>
  </EvidenceDisclosure>
));

export const sessionToolView = view("session-tool", "tool", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-tool");
  return (
    <EvidenceDisclosure className="chat-step chat-step--tool" label={block?.name ?? "tool"} children={null} value={null} summary={<SessionReading label="recorded tool summary" value={{ name: block?.name ?? slot.text.slice(0, 40), args: (block?.args ?? []).map(([name, value]) => [name, value.length > 60 ? `${value.slice(0, 60)}…` : value]) }}>
        <code>{block?.name ?? slot.text.slice(0, 40)}</code>
        {(block?.args ?? []).map(([k, v]) => (
          <span key={k} className="chat-step__arg">
            {" "}
            {k}=<code>{v.length > 60 ? `${v.slice(0, 60)}…` : v}</code>
          </span>
        ))}
      </SessionReading>}/>
  );
});

export const sessionToolResultView = view("session-tool-result", "tool-result", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-tool-result");
  return (
    <EvidenceDisclosure className="chat-step chat-step--tool" label="result" value={block?.body ?? slot.text.trim()} summary={<SessionReading label="recorded result summary" value={{ name: block?.name ?? null, failed: block?.ok === false }}>
        result
        {block?.name ? (
          <>
            {" "}
            of <code>{block.name}</code>
          </>
        ) : null}
        {block?.ok === false ? <span className="chat-error"> failed</span> : null}
      </SessionReading>}>
      <pre>{block?.body ?? slot.text.trim()}</pre>
    </EvidenceDisclosure>
  );
});

export const sessionInputView = view("session-input", "input", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-input");
  return (
    <EvidenceDisclosure className="chat-step chat-step--tool" label="input" value={block?.body ?? slot.text.trim()} summary={<SessionReading label="recorded input summary" value={block?.name ?? null}>input{block?.name ? <> · {block.name}</> : null}</SessionReading>}>
      <pre>{block?.body ?? slot.text.trim()}</pre>
    </EvidenceDisclosure>
  );
});

export const sessionReadView = view("session-read", "read", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-read");
  return (
    <SessionReading label="recorded read metadata" value={{ file: block?.file ?? null, lines: block?.lines ?? null, sha256: block?.sha256 ?? null, commit: block?.commit ?? null }}><p
      className="chat-step chat-step--read muted"
      data-tip={
        block?.sha256 ? `sha256 ${block.sha256}${block.commit ? ` · commit ${block.commit}` : ""}` : undefined
      }
    >
      read <code>{block?.file ?? "a file"}</code>
      {block?.lines ? <> lines {block.lines}</> : null}
    </p></SessionReading>
  );
});

export const sessionWroteView = view("session-wrote", "wrote", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-wrote");
  return (
    <SessionReading label="recorded wrote metadata" value={{ file: block?.file ?? null, lines: block?.lines ?? null }}><p className="chat-step chat-step--wrote muted">
      wrote <code>{block?.file ?? "a file"}</code>
      {block?.lines ? <> lines {block.lines}</> : null}
    </p></SessionReading>
  );
});

export const sessionContextView = view("session-context", "context", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-context");
  if ((block?.context_kind?.startsWith("acp-") || block?.context_kind?.startsWith("filesystem-"))) return <AcpRecord kind={block.context_kind} body={block.body ?? slot.text.trim()} />;
  return (
    <EvidenceDisclosure className="chat-step chat-step--context" label="context" value={block?.body ?? slot.text.trim()} summary={<SessionReading label="recorded context summary" value={block?.context_kind ?? null}>context{block?.context_kind ? <> · {block.context_kind}</> : null}</SessionReading>}>
      <pre>{block?.body ?? slot.text.trim()}</pre>
    </EvidenceDisclosure>
  );
});

export const sessionObservationView = view("session-observation", "observation", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-observation");
  return (
    <EvidenceDisclosure className="chat-step chat-step--tool" label="observation" value={block?.body ?? slot.text.trim()} summary={<SessionReading label="recorded observation summary" value={block?.exit ?? null}>observation{block?.exit ? <> · exit {block.exit}</> : null}</SessionReading>}>
      <pre>{block?.body ?? slot.text.trim()}</pre>
    </EvidenceDisclosure>
  );
});

export const sessionActionView = view("session-action", "action", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-action");
  return (
    <EvidenceDisclosure className="chat-step chat-step--action" label="ran" value={block?.body ?? slot.text.trim()} summary={<SessionReading label="recorded ran summary" value={block?.lang ?? null}>ran{block?.lang ? <> · {block.lang}</> : null}</SessionReading>}>
      <pre>{block?.body ?? slot.text.trim()}</pre>
    </EvidenceDisclosure>
  );
});

INNER["session-reasoning"] = sessionReasoningView.render;
INNER["session-tool"] = sessionToolView.render;
INNER["session-action"] = sessionActionView.render;
INNER["session-input"] = sessionInputView.render;

export const sessionViews: ElementView[] = [
  sessionUserView,
  sessionAssistantView,
  sessionReasoningView,
  sessionToolView,
  sessionToolResultView,
  sessionInputView,
  sessionReadView,
  sessionWroteView,
  sessionContextView,
  sessionObservationView,
  sessionActionView,
  {
    kind: "session-meta",
    draws: (block) => block.name === "usage" || block.name === "next",
    render: () => <span className="session-meta" aria-hidden />,
  },
];
</book:entry>

</book:doc>
