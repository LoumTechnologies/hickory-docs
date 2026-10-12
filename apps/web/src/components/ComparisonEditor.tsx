import { useEffect, useRef, useState } from "react";
import { EditorState, Compartment } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { languageExtensions, languageFromPath } from "../editor/languages";
import { comparisonField, setComparison } from "../editor/comparison";
import { editorChrome } from "../editor/chrome";
import { canEdit, comparisonDiff, editComparison, forkComparison, type ComparisonModel, type ComparisonSource } from "../lib/projectComparison";
import { ComparisonTools } from "./ComparisonTools";
import { DiffView } from "./DiffView";

function SideEditor({ path, item, base, editable, onChange, compact }: {
  path: string; item: ComparisonSource; base?: string; editable: boolean; onChange: (source: string) => void; compact?: boolean;
}) {
  const {label,text:source} = item;
  const [mounted,setMounted] = useState<EditorView|null>(null);
  const host = useRef<HTMLDivElement>(null), view = useRef<EditorView | null>(null);
  const changed = useRef(onChange); changed.current = onChange;
  const access = useRef(new Compartment());
  const synchronizing = useRef(false);
  useEffect(() => {
    const language = languageFromPath(path);
    const editor = new EditorView({ parent: host.current!, state: EditorState.create({ doc: source, extensions: [
      editorChrome("document"), history(), keymap.of([...defaultKeymap, ...historyKeymap]),
      ...(language ? languageExtensions(language) : []), comparisonField,
      access.current.of([EditorState.readOnly.of(!editable), EditorView.editable.of(editable)]),
      EditorView.contentAttributes.of({ "aria-label": label }),
      EditorView.updateListener.of(update => { if (update.docChanged && !synchronizing.current) changed.current(update.state.doc.toString()); }),
    ] }) });
    view.current = editor; setMounted(editor);
    return () => { editor.destroy(); view.current = null; };
  }, [path, label]);
  useEffect(() => {
    const editor = view.current; if (!editor) return;
    synchronizing.current = true;
    try {
      editor.dispatch({ ...(editor.state.doc.toString() !== source ? { changes: { from: 0, to: editor.state.doc.length, insert: source } } : {}),
        effects: [access.current.reconfigure([EditorState.readOnly.of(!editable), EditorView.editable.of(editable)]),
          setComparison.of({ base: base ?? null, editable })] });
    } finally { synchronizing.current = false; }
  }, [source, base, editable]);
  return <section className="comparison-side">{!compact && <h3>{label} · {editable ? "editable draft" : "read-only"}</h3>}<div ref={host} />{item.project && <ComparisonTools path={path} source={item} view={mounted} />}</section>;
}

/** One N-source model; two chosen endpoints, with independent storage and evidence. */
export function ComparisonEditor({ path, model, onChange, allowFork = false, compact = false }: {
  path: string; model: ComparisonModel; onChange?: (model: ComparisonModel) => void; allowFork?: boolean; compact?: boolean;
}) {
  const [layout, setLayout] = useState("inline");
  const old = model.sources.find(s => s.id === model.old)!, next = model.sources.find(s => s.id === model.new)!;
  const change = (side: "old" | "new", text: string) => onChange?.(editComparison(model, side, text));
  return <section className="comparison-editor" aria-label={`Comparison of ${path}`}>
    <header className="doc-tab-toolbar">
      {!compact && <span>{old.label} → {next.label} · {model.evidence === "verified" ? "output equality verified" : "unverified"}</span>}
      <label>Layout <select aria-label="Comparison layout" value={layout} onChange={e => setLayout(e.target.value)}>
        <option value="inline">Inline</option><option value="side-by-side">Side by side</option><option value="patch">Patch</option>
      </select></label>
      {allowFork && model.policy === "neither" && <button className="btn" onClick={() => onChange?.(forkComparison(model))}>Fork editable comparison</button>}
      {model.sources.length > 2 && <label>New source <select value={model.new} onChange={e => onChange?.({ ...model, new: e.target.value, policy: "neither", evidence: "unverified" })}>
        {model.sources.filter(s => s.id !== model.old).map(s => <option key={s.id} value={s.id}>{s.label}</option>)}
      </select></label>}
    </header>
    {layout === "patch" ? <DiffView path={path} diff={comparisonDiff(path, old.text, next.text)} binary={false} staged={false} label="Local transition" />
      : <div className={`comparison-panes comparison-panes--${layout}`}>
        {(layout === "side-by-side" || canEdit(model, "old")) && <SideEditor path={old.path ?? path} item={old} compact={compact}
          editable={!!onChange && canEdit(model, "old")} onChange={text => change("old", text)} />}
        <SideEditor path={next.path ?? path} item={next} compact={compact} base={layout === "inline" ? old.text : undefined}
          editable={!!onChange && canEdit(model, "new")} onChange={text => change("new", text)} />
      </div>}
  </section>;
}
