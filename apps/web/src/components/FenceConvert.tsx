// "Make this fence a cell" — the panel a fence's rail icon opens.
//
// It shows the exec cell it would write before writing it, for the same
// reason the Insert menu does: this is a text format a person owns, and a
// conversion you cannot see is one you cannot check. The container is the
// only choice worth asking about, because it is the only thing the fence
// cannot tell us — everything else is derived from the language tag.

import { useMemo, useState } from "react";

import { convertFence } from "../lib/fenceToExec";
import { elementById, initialValues, renderElement } from "../lib/insertCatalog";

export interface FenceConvertProps {
  /** The fence's info string and body. */
  info: string;
  body: string;
  /** Containers this document already declares, in document order. */
  containers: string[];
  /** Replace the fence with this text. */
  onConvert: (text: string) => void;
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
          onChange={(event) => setContainer(event.target.value)}
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
          onChange={(event) => setContainer(event.target.value)}
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

      <div className="insert-menu__actions">
        <button type="button" className="btn" onClick={onCancel}>
          Cancel
        </button>
        <button
          type="button"
          className="btn btn-primary"
          disabled={!container.trim()}
          data-tip={container.trim() ? undefined : "Name the container it runs in"}
          onClick={() => onConvert(preview)}
        >
          Convert
        </button>
      </div>
    </div>
  );
}
