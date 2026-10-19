// Acceptance host: product review UI and attribution on the supported document embed.
import { providerObservationContext } from "../debug/providerObservations";
import { useEffect, useMemo, useState } from "react";
import type { EditorView } from "@codemirror/view";
import { DocumentEmbed } from "./DocumentEmbed";
import { ReviewPanel } from "../components/ReviewPanel";
import { LineageSettings } from "../components/LineageSettings";
import { installAttributionProvider } from "../lib/attribution";
import { blameGutter } from "../editor/blameGutter";
import { useBlame } from "../editor/useBlame";
import "../styles.css";
export function ReviewFixture() {
  const [githubContext, setGithubContext] = useState("");
  const [view, setView] = useState<EditorView | null>(null);
  const [source, setSource] = useState("Human requirement\nAgent claim about total deadline\n");
  const extensions = useMemo(() => [blameGutter()], []);
  useEffect(() => installAttributionProvider({ id: "fixture-receipts", label: "Recorded roles", lines: async () => [
    {line:1,text:"Human requirement",author:"Sam",email:"sam@example.com",commit:"one",time:0,summary:"",uncommitted:false,origin:"human",provider:"Recorded role"},
    {line:2,text:"Agent claim about total deadline",author:"Sam",email:"sam@example.com",commit:"one",time:0,summary:"",uncommitted:false,origin:"agent",provider:"Recorded agent write"},
  ] }), []);
  useBlame(view, "spec.md");
  return <main style={{padding:24}}><h1>Review acceptance workspace</h1><ReviewPanel document="spec.md" source={source} view={view} /><LineageSettings />
    <button onClick={() => setGithubContext(JSON.stringify(providerObservationContext("spec.md", source)))}>Inspect selected GitHub context</button><output aria-label="Selected GitHub context">{githubContext}</output>
    <DocumentEmbed path="spec.md" revision={source} source={source} onChange={change => setSource(change.source)} onViewReady={setView} extensions={extensions} />
  </main>;
}
