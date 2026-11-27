import { useLayoutEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { loadHickLang } from "../editor/hickLang";
import {
  elementOptions, newElement, planElement, readElements,
  type ElementOption, type ElementPlan, type ElementReading, type ElementTarget,
} from "./ElementAuthoringModel";
import "./ElementAuthoring.css";

type FormProps = {
  reading: ElementReading;
  range: { from: number; to: number };
  targets?: ElementTarget[];
  close: () => void;
  apply: (plan: ElementPlan) => string | undefined;
};
const unavailable = "The document parser could not load. Reopen the document to retry.";

export function ElementAuthoring({ reading, range, targets, close, apply }: FormProps) {
  const [target, setTarget] = useState(targets?.length === 1 ? targets[0] : undefined);
  const [option, setOption] = useState<ElementOption | undefined>(target?.option);
  const [query, setQuery] = useState("");
  const [failure, setFailure] = useState("");
  const editing = !!targets;
  const title = editing ? "Edit element" : "Insert element";
  const results = elementOptions.filter(item => query.trim().toLowerCase().split(/\s+/)
    .every(word => [item.title, item.id, item.group, item.summary].join(" ").toLowerCase().includes(word)));
  const matches = targets?.map(item => item.option) ?? results;
  const origin = target ?? range;
  const selected = reading.source.slice(origin.from, origin.to);

  return <section className="element-authoring" aria-label={title}>
    <header><strong>{title}</strong><button type="button" aria-label="Close element form" onClick={close}>×</button></header>
    {option ? <>
      {(targets?.length !== 1) && <button className="element-back" type="button" onClick={() => { setTarget(undefined); setOption(undefined); setFailure(""); }}>← Choose an element</button>}
      <ElementFields key={option.id + ":" + origin.from} option={option} target={target} selected={selected}
        plan={(values, body) => planElement(reading, origin, option, values, body, target)}
        apply={plan => setFailure(apply(plan) ?? "")} />
    </> : <>
      {!editing && <input onKeyDown={event => {
        if (event.key === "Enter" && !event.nativeEvent.isComposing && results.length) { event.preventDefault(); setOption(results[0]); }
      }} className="element-search" aria-label="Find an element" placeholder="Find an element" value={query} onChange={e => setQuery(e.target.value)} autoFocus />}
      <div className="element-options">
        {matches.map((item, index) => <button type="button" key={item.id + ":" + index} onClick={() => {
          setTarget(targets?.[index]); setOption(item);
        }}>
          <span>{item.title}</span>
          <small>{editing ? reading.source.slice(targets![index].from, targets![index].to).split(/\r?\n/)[0] : item.group}</small>
        </button>)}
        {!matches.length && <p>No matching elements.</p>}
      </div>
    </>}
    {failure && <p role="alert">{failure}</p>}
  </section>;
}

function ElementFields({ option, target, selected, plan, apply }: {
  option: ElementOption; target?: ElementTarget; selected: string;
  plan: (values: Record<string, string>, body: string) => ElementPlan;
  apply: (plan: ElementPlan) => void;
}) {
  const [initial] = useState(() => target ?? newElement(option, selected));
  const [values, setValues] = useState(initial.values);
  const [body, setBody] = useState(initial.body);
  const [bodyEdited, setBodyEdited] = useState(false);
  const first = useRef<HTMLElement | null>(null);
  useLayoutEffect(() => first.current?.focus(), []);
  let preview: ElementPlan | undefined;
  let error = "";
  try { preview = plan(values, body); }
  catch (failure) { error = failure instanceof Error ? failure.message : String(failure); }
  const changed = preview?.insert !== selected;
  const update = (name: string, value: string) => {
    const next = { ...values, [name]: value };
    setValues(next);
    if (!target && !bodyEdited) setBody(newElement(option, selected, next).body);
  };
  return <form onSubmit={event => { event.preventDefault(); if (preview && changed) apply(preview); }}>
    <h2>{option.title}</h2>
    {option.summary && <p>{option.summary}</p>}
    {option.belongsIn && <p>Place inside <code>{option.belongsIn}</code>.</p>}
    {option.fields.map((field, index) => {
      const value = values[field.name] ?? "";
      const multiline = option.group !== "Markdown" &&
        (/[\r\n]/.test(initial.values[field.name] ?? "") || ["sources", "exclude"].includes(field.name));
      return <label key={field.name}>
        <span>{field.label}{option.group !== "Markdown" && <> <code>({field.name})</code></>}</span>
        {field.choices ? <select value={value} onChange={event => update(field.name, event.target.value)}
          ref={node => { if (!index) first.current = node; }}>
          {!field.choices.includes(value) && <option value={value}>{value || "Default"}</option>}
          {field.choices.map(choice => <option key={choice} value={choice}>{choice || "Default"}</option>)}
        </select> : multiline ? <textarea value={value} placeholder={field.placeholder} ref={node => { if (!index) first.current = node; }}
          onChange={event => update(field.name, event.target.value)} /> :
          <input value={value} placeholder={field.placeholder} ref={node => { if (!index) first.current = node; }}
            onChange={event => update(field.name, event.target.value)} />}
        {field.hint && <small>{field.hint}</small>}
      </label>;
    })}
    {option.bodyLabel && <label>{option.bodyLabel}
      <textarea value={body} spellCheck={false} ref={node => { if (!option.fields.length) first.current = node; }}
        onChange={event => { setBodyEdited(true); setBody(event.target.value); }} />
    </label>}
    {error && <p role="status">{error}</p>}
    {preview && <div><strong>Source preview</strong><pre aria-label="Source preview">{preview.insert}</pre></div>}
    <button className="element-apply" type="submit" disabled={!preview || !changed}>{target ? "Apply edit" : "Insert"}</button>
  </form>;
}

export function ElementDialog({ anchor, ...props }: FormProps & { anchor?: HTMLElement }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const position = anchor?.getBoundingClientRect();
  useLayoutEffect(() => {
    const node = dialog.current!;
    node.showModal();
    return () => node.close();
  }, []);
  return <dialog className="element-dialog" ref={dialog} aria-label={props.targets ? "Edit element" : "Insert element"}
    style={position ? { margin: 0, right: Math.max(8, Math.min(window.innerWidth - Math.min(360, window.innerWidth - 24) - 8, window.innerWidth - position.left + 6)), left: "auto", top: Math.max(8, Math.min(position.top, window.innerHeight - 490)) } : undefined}
    onCancel={event => { event.preventDefault(); props.close(); }}
    onClick={event => { if (event.target === event.currentTarget) props.close(); }}>
    <ElementAuthoring {...props} />
  </dialog>;
}

type NativeOpen = { id: string; revision: string; from: number; to: number; line?: number };
type NativeMessage =
  | { kind: "ready" }
  | { kind: "cancel"; id: string }
  | { kind: "lines"; revision: string; lines: number[] }
  | { kind: "apply"; id: string; plan: ElementPlan }
  | { kind: "error"; message: string };
export function mountNativeElements(container: HTMLElement, send: (message: NativeMessage) => void) {
  const root = createRoot(container);
  let reading: ElementReading | undefined, revision = "";
  const ready = loadHickLang();
  void ready.then(() => send({ kind: "ready" }), () => send({ kind: "error", message: unavailable }));
  return {
    async read(next: string, source: string) {
      revision = next;
      reading = undefined;
      await ready.catch(() => undefined);
      if (next !== revision) return;
      try {
        reading = readElements(source);
        send({ kind: "lines", revision, lines: [...elementsByLine(reading).keys()] });
      } catch {
        reading = undefined;
        send({ kind: "error", message: unavailable });
      }
    },
    open(request: NativeOpen) {
      if (!reading || revision !== request.revision) {
        send({ kind: "error", message: "The element reading changed. Open the form again." });
        return;
      }
      const targets = request.line === undefined ? undefined : elementsByLine(reading).get(request.line);
      if (request.line !== undefined && !targets?.length) {
        send({ kind: "error", message: "That line changed. Choose its current edit icon." });
        return;
      }
      root.render(<ElementAuthoring key={request.id} reading={reading} range={request} targets={targets} close={() => { root.render(null); send({ kind: "cancel", id: request.id }); }}
        apply={plan => { send({ kind: "apply", id: request.id, plan }); return undefined; }} />);
    },
    close() { root.render(null); },
  };
}

export function elementsByLine(reading: ElementReading): Map<number, ElementTarget[]> {
  const lines = new Map<number, ElementTarget[]>();
  const breaks = reading.source.matchAll(/\r\n?|\n/g);
  let line = 0, next = breaks.next().value;
  for (const target of reading.targets) {
    while (next && next.index + next[0].length <= target.from) { line++; next = breaks.next().value; }
    const items = lines.get(line) ?? [];
    items.push(target);
    lines.set(line, items);
  }
  return lines;
}
