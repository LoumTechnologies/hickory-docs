// "The code in a document is code" — demonstrated rather than asserted.
//
// A reader looking at a screenshot of a `.md` document cannot tell whether
// the TypeScript inside it is understood or merely syntax-highlighted. So
// this runs the actual TypeScript compiler in their browser, over the block
// in the document below, through the SAME editor bindings the desktop app
// uses: hover, completion, diagnostics and semantic colouring.
//
// Break the code and the squiggle appears, because the compiler ran. That is
// the only version of this demo worth shipping.

import { useEffect, useMemo, useRef, useState } from "react";
import type { Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { LspClient } from "../../lsp/client";
import {
  diagnosticRanges,
  lspSupport,
  offsetToPosition,
  positionToOffset,
  setLspDiagnostics,
} from "../../lsp/cmLsp";
import { lspFeatures } from "../../lsp/cmLspFeatures";
import { createBrowserTsChannel } from "./browserTsServer";
import { DemoEditor } from "./DemoEditor";

/**
 * The document the demo opens.
 *
 * Chosen so that one obvious edit — deleting the `?? 0`, or passing a string
 * where a number belongs — produces a real type error, because the fastest
 * way to believe a compiler is running is to break something and watch it
 * notice.
 */
const DOCUMENT = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="pricing.md">
# How a cart is priced

Every line is quantity times unit price; a missing discount counts as none.

<hick:file path="pricing.ts">
interface Line {
  quantity: number;
  unitPrice: number;
  discount?: number;
}

export function lineTotal(line: Line): number {
  const discount = line.discount ?? 0;
  return line.quantity * line.unitPrice * (1 - discount);
}

export function cartTotal(lines: Line[]): number {
  return lines.reduce((sum, line) => sum + lineTotal(line), 0);
}
</hick:file>
</hick:doc>
`;

const URI = "hick:///pricing.md";

export function IntelligenceDemo() {
  const [source, setSource] = useState(DOCUMENT);
  // The compiler is ~1 MB gzipped. A reader who never scrolls this far must
  // never pay for it, so nothing is fetched until the demo is on screen —
  // and the editor is fully usable, and correctly highlighted, meanwhile.
  const [visible, setVisible] = useState(false);
  const [status, setStatus] = useState<"idle" | "loading" | "ready" | "failed">("idle");
  const [message, setMessage] = useState<string | null>(null);
  const [problems, setProblems] = useState<string[]>([]);
  const viewRef = useRef<EditorView | null>(null);
  // The buffer, held in a ref: the channel needs the latest text on every
  // request, and re-creating the client on each keystroke would restart the
  // compiler.
  const sourceRef = useRef(source);
  sourceRef.current = source;

  const frameRef = useRef<HTMLElement>(null);
  useEffect(() => {
    const element = frameRef.current;
    if (!element) return;
    // No IntersectionObserver (jsdom, older browsers): load immediately
    // rather than leaving the demo permanently inert.
    if (typeof IntersectionObserver !== "function") {
      setVisible(true);
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          setVisible(true);
          observer.disconnect();
        }
      },
      { rootMargin: "200px" },
    );
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const client = useMemo(() => {
    if (!visible) return null;
    const channel = createBrowserTsChannel({
      document: DOCUMENT,
      onReady: () => setStatus("ready"),
      onError: (why) => {
        setStatus("failed");
        setMessage(why);
      },
    });
    return { client: new LspClient(channel), channel };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [visible]);

  // Diagnostics arrive as a notification, exactly as they do from a real
  // server, and are drawn by the product's own decoration field.
  useEffect(() => {
    if (!client) return;
    setStatus("loading");
    const stop = client.client.onDiagnostics((params) => {
      setProblems(params.diagnostics.map((diagnostic) => diagnostic.message));
      const view = viewRef.current;
      if (!view) return;
      const ranges = diagnosticRanges(params.diagnostics, (position) =>
        positionToOffset(view.state.doc, position),
      );
      view.dispatch({ effects: setLspDiagnostics.of(ranges) });
    });
    client.client.request("initialize", {}).catch(() => undefined);
    client.client.didOpen(URI, DOCUMENT);
    return () => {
      stop();
      client.client.dispose();
      client.channel.dispose();
    };
  }, [client]);

  const extensions: Extension[] = useMemo(
    () =>
      client === null
        ? []
        : [
      ...lspSupport({
        client: client.client,
        uri: URI,
        positionAt: (offset, view) => offsetToPosition(view.state.doc, offset),
        // Nothing to navigate to on a marketing page: the definition is
        // already on screen, so this scrolls to it rather than opening
        // anything.
        onNavigate: (target) => {
          const view = viewRef.current;
          if (!view) return;
          const offset = positionToOffset(view.state.doc, target.range.start);
          view.dispatch({ selection: { anchor: offset }, scrollIntoView: true });
        },
      }),
      ...lspFeatures({
        client: client.client,
        uri: URI,
        positionAt: (offset, view) => offsetToPosition(view.state.doc, offset),
        offsetAt: (position, view) => positionToOffset(view.state.doc, position),
        inlayHints: false,
      }),
    ],
    [client],
  );

  return (
    <figure className="intel-demo" data-testid="intelligence-demo" ref={frameRef}>
      <figcaption className="intel-demo__caption">
        <strong>The code in a document is code.</strong> Hover a name, type a{" "}
        <code>.</code>, or break something — the TypeScript compiler is running on this page,
        answering through the same editor the app ships.
      </figcaption>

      <DemoEditor
        value={source}
        onChange={(next) => {
          setSource(next);
          client?.channel.update(next);
          client?.client.didChange(URI, next, Date.now());
        }}
        hick
        extraExtensions={extensions}
        onViewReady={(view) => {
          viewRef.current = view;
        }}
        ariaLabel="A hick document with TypeScript in it"
        testId="intelligence-editor"
        className="intel-demo__editor"
      />

      <p className="intel-demo__status" role="status">
        {status === "idle" && "Scroll here and the compiler starts."}
        {status === "loading" && "Loading the TypeScript compiler…"}
        {status === "failed" && (message ?? "The compiler could not be loaded.")}
        {status === "ready" && problems.length === 0 && "No problems. Try breaking something."}
        {status === "ready" &&
          problems.length > 0 &&
          `${problems.length} problem${problems.length === 1 ? "" : "s"}: ${problems[0]}`}
      </p>

      <p className="intel-demo__note">
        This page runs TypeScript because TypeScript's compiler happens to run in a browser. In
        the app it is whichever language servers your machine already has — for every language
        your document contains, found without configuring anything.
      </p>
    </figure>
  );
}
