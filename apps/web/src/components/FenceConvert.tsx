// "Make this fence a cell" — the panel a fence's rail icon opens.
//
// It shows the exec cell it would write before writing it, for the same
// reason the Insert menu does: this is a text format a person owns, and a
// conversion you cannot see is one you cannot check. The container is the
// only choice worth asking about, because it is the only thing the fence
// cannot tell us — everything else is derived from the language tag.

import { useId, useMemo, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";
import { convertFence } from "../lib/fenceToExec";
import { elementById, initialValues, renderElement } from "../lib/insertCatalog";

export interface FenceConvertProps {
  /** The fence's info string and body. */
  info: string;
  body: string;
  /** Containers this document already declares, in document order. */
  containers: string[];
  /** Replace the fence with this text. */
  onConvert: (text: string) => boolean | void;
  onCancel: () => void;
}

export function FenceConvert({
  info,
  body,
  containers,
  onConvert,
  onCancel,
}: FenceConvertProps) {
  // The first declared container is nearly always the right answer, and a
  // document with none gets a name to type rather than a silent omission —
  // an exec with no container and no image has nowhere to run.
  const [container, setContainer] = useState(containers[0] ?? "");
  const conversion = useMemo(() => convertFence({ info, body }), [info, body]);

  const exec = elementById("exec");
  const preview = useMemo(() => {
    if (!exec) return "";
    return renderElement(exec, { ...initialValues(exec), container }, conversion.body);
  }, [exec, container, conversion.body]);

  const parent = `fence-convert:${useId()}`;
  const [error, setError] = useState<string | null>(null);
  const callbackRevision = useMemo(() => crypto.randomUUID(), [onConvert, onCancel]);
  const revision = useUiRevision(() => [info, body, containers, container, preview, callbackRevision]);
  const changeContainer = (text: string): UiOutcome => {
    if (text.length > 256 || (containers.length > 0 && !containers.includes(text))) return { status: "unavailable", reason: "Choose a currently declared container" };
    setContainer(text); setError(null);
    return { status: "completed", value: { draft_updated: true } };
  };
  const convert = (): UiOutcome => {
    if (!container.trim() || (containers.length > 0 && !containers.includes(container))) return { status: "unavailable", reason: "Choose a current container before converting" };
    try {
      const changed = onConvert(preview);
      if (changed === false) {
        const reason = "The fence source or editor authority changed. Reopen this fence and inspect the current preview.";
        setError(reason); return { status: "requires_input", request: `${parent}:source`, reason };
      }
      return { status: "completed", value: { edit_requested: true, source_changed: changed === true, saved: false, executed: false } };
    } catch (cause) { setError(String(cause)); return { status: "unavailable", reason: String(cause) }; }
  };
  const cancel = (): UiOutcome => { onCancel(); return { status: "completed", value: { dismissal_requested: true, converted: false } }; };
  useUiCommands([
    { key: `${parent}:container`, command: { parent, subject: parent, label: "Set fence conversion container", parameters: { container: { type: "string", maxLength: 256, ...(containers.length ? { choices: containers } : {}) } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async args => changeContainer(args.container as string) } },
    { key: `${parent}:convert`, command: { parent, subject: parent, label: "Convert fence to executable cell", parameters: {}, prerequisites: ["Preview is a source edit, not execution or saving"],
      read: () => ({ revision: revision(), enabled: !!container.trim() && (!containers.length || containers.includes(container)) }), execute: async () => convert() } },
    { key: `${parent}:cancel`, command: { parent, subject: parent, label: "Cancel fence conversion", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async () => cancel() } },
  ]);
  useUiReading("fence conversion preview", { info, body, container, containers, conversion, preview, error, scope: "Rendered preview; conversion neither saves nor executes the cell" });

  return (
    <div className="fence-convert">
      <header className="fence-convert__header">
        <h3 className="fence-convert__title">Make this a cell</h3>
        <span className="fence-convert__lang mono">
          {conversion.language || "no language tag"}
        </span>
      </header>

      <p className="fence-convert__how">
        {conversion.how === "heredoc"
          ? "The program is fed to its interpreter on standard input, so the cell runs exactly this code."
          : conversion.how === "verbatim"
            ? "The fence is already shell, so its commands transfer unchanged."
            : "Carried across unchanged — see below."}
      </p>

      <label className="fence-convert__label" htmlFor="fence-convert-container">
        Container
        <span className="mono insert-menu__attr">container</span>
      </label>
      {containers.length > 0 ? (
        <select
          id="fence-convert-container"
          className="insert-menu__input"
          value={container}
          onChange={(event) => changeContainer(event.target.value)}
        >
          {containers.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
      ) : (
        <input
          id="fence-convert-container"
          className="insert-menu__input"
          type="text"
          value={container}
          spellCheck={false}
          placeholder="py"
          maxLength={256}
          onChange={(event) => changeContainer(event.target.value)}
        />
      )}
      {containers.length === 0 && (
        <p className="insert-menu__hint">
          This document declares no container yet. Name one here, then add a{" "}
          <span className="mono">hick:container</span> for it — the Insert menu has one.
        </p>
      )}

      {conversion.note && (
        <p className="fence-convert__note" role="note">
          {conversion.note}
        </p>
      )}

      <div className="insert-menu__preview">
        <h4 className="insert-menu__preview-title">What replaces the fence</h4>
        <pre className="insert-menu__preview-text mono">{preview}</pre>
      </div>

      {error && <p role="alert">{error}</p>}
      <div className="insert-menu__actions">
        <button type="button" className="btn" onClick={() => cancel()}>
          Cancel
        </button>
        <button
          type="button"
          className="btn btn-primary"
          disabled={!container.trim() || (containers.length > 0 && !containers.includes(container))}
          data-tip={container.trim() ? undefined : "Name the container it runs in"}
          onClick={() => convert()}
        >
          Convert
        </button>
      </div>
    </div>
  );
}
