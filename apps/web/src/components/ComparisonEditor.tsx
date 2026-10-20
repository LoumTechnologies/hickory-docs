import { ComparisonControls } from "./ComparisonControls";
import { useComparisonDraftActions } from "./useComparisonDraftActions";
import { useEditorActions } from "../editor/uiEditorActions";
import { isDebuggable } from "../debug/languages";
import { DebugLaunchRail, DebugLaunchAction } from "../debug/DebugRail";
import { RightRail } from "../editor/RightRail";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { EditorState, Compartment } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { languageExtensions, languageFromPath } from "../editor/languages";
import { completions } from "../lsp/completion";
import { comparisonField, setComparison, type ComparisonState } from "../editor/comparison";
import { editorChrome } from "../editor/chrome";
import { canEdit, comparisonDiff, editComparison, type ComparisonModel, type ComparisonSource } from "../lib/projectComparison";
import { ComparisonTools } from "./ComparisonTools";
import { DiffView } from "./DiffView";
import { MarginActions } from "./MarginActions";

export function CodeEditor({ path, item, base, presentation, editable, onChange, compact }: {
  path: string; item: ComparisonSource; base?: string; presentation?: ComparisonState["presentation"]; editable: boolean; onChange: (source: string) => void; compact?: boolean;
}) {
  const {label,text:source} = item;
  const [mounted,setMounted] = useState<EditorView|null>(null);
  const host = useRef<HTMLDivElement>(null), view = useRef<EditorView | null>(null);
  useEditorActions(mounted, `comparison:${item.id}:${path}`);
  useComparisonDraftActions(mounted, editable, JSON.stringify({ id: item.id, path, kind: item.kind, target: item.target, code_uri: item.codeUri }));
  const changed = useRef(onChange); changed.current = onChange;
  const access = useRef(new Compartment());
  const synchronizing = useRef(false);
  useEffect(() => {
    const language = languageFromPath(path);
    const editor = new EditorView({ parent: host.current!, state: EditorState.create({ doc: source, extensions: [
      editorChrome("code"), history(), keymap.of([...defaultKeymap, ...historyKeymap]),
      ...(language ? languageExtensions(language) : []), comparisonField, completions({}),
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
          setComparison.of({ base: base ?? null, editable, presentation })] });
    } finally { synchronizing.current = false; }
  }, [source, base, editable, presentation]);
  useEffect(() => {
    if (!mounted || !item.location) return;
    const line = mounted.state.doc.line(Math.min(item.location.line + 1, mounted.state.doc.lines));
    mounted.dispatch({selection:{anchor:Math.min(line.to, line.from + item.location.character)},scrollIntoView:true});
    mounted.focus();
  }, [mounted, item.location]);
  return <section className="comparison-side">{!compact && <h3>{label} · {editable ? "editable draft" : "read-only"}</h3>}<div className="comparison-code with-right-rail"><div className="editor-cm-host" ref={host} /><RightRail view={mounted} /><DebugLaunchRail view={mounted} /></div>{item.project ? <ComparisonTools path={path} source={item} view={mounted} /> : isDebuggable(languageFromPath(path)) && item.kind === "authored" && <DebugLaunchAction view={mounted} label={`Debug ${path}`} onStart={() => {}} disabled hint="Save or apply this draft before debugging its project." />}</section>;
}

/** One N-source model; two chosen endpoints, with independent storage and evidence. */
export function ComparisonEditor({ path, model, onChange, allowFork = false, compact = false, options }: {
  path: string; model: ComparisonModel; onChange?: (model: ComparisonModel) => void; allowFork?: boolean; compact?: boolean; options?: ReactNode;
}) {
  const [layout, setLayout] = useState("side-by-side");
  const old = model.sources.find(s => s.id === model.old)!, next = model.sources.find(s => s.id === model.new)!;
  const change = (side: "old" | "new", text: string) => onChange?.(editComparison(model, side, text));
  const controls = <ComparisonControls path={path} model={model} layout={layout} setLayout={setLayout} onChange={onChange} allowFork={allowFork} compact={compact} options={options} />;
  return <section className={`comparison-editor${compact ? " comparison-editor--compact" : ""}`} aria-label={`Comparison of ${path}`}>
    {compact ? <MarginActions label="Patch options" glyph="⋯">{controls}</MarginActions> : <header className="doc-tab-toolbar">{controls}</header>}
    {layout === "patch" ? <ComparisonTools path={old.path ?? path} source={old} view={null}>{oldDebug =>
      <ComparisonTools path={next.path ?? path} source={next} view={null}>{newDebug =>
        <DiffView path={path} diff={comparisonDiff(path, old.text, next.text)} binary={false} staged={false} label="Local transition"
          gutter={(side, line) => {
            const debug = side === "old" ? oldDebug : newDebug;
            if (!debug) return null;
            const held = debug.breakpoints.find(bp => bp.line === line);
            return <button type="button" className="diff-breakpoint" aria-label={`${held ? "Remove" : "Set"} breakpoint on ${side} line ${line + 1}`}
              aria-pressed={!!held} data-tip={held?.message ?? `${side === "old" ? old.label : next.label} · line ${line + 1}`}
              onClick={() => debug.toggleBreakpoint(line)}>{held ? "●" : "○"}</button>;
          }} />
      }</ComparisonTools>
    }</ComparisonTools>
      : <div className={`comparison-panes comparison-panes--${layout}`}>
        {(layout === "side-by-side" || canEdit(model, "old")) && <CodeEditor path={old.path ?? path} item={old} compact={compact} base={layout === "side-by-side" ? next.text : undefined} presentation="old"
          editable={!!onChange && canEdit(model, "old")} onChange={text => change("old", text)} />}
        <CodeEditor path={next.path ?? path} item={next} compact={compact} base={old.text} presentation={layout === "side-by-side" ? "new" : "inline"}
          editable={!!onChange && canEdit(model, "new")} onChange={text => change("new", text)} />
      </div>}
  </section>;
}
