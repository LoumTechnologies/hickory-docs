import type { TemplateOption } from "../api/types";
import { NO_RESTORE } from "../lib/scaffold";

export function Field({
  label,
  attr,
  hint,
  problem,
  children,
}: {
  label: string;
  attr: string;
  hint: string;
  problem?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="insert-menu__field">
      <label className="insert-menu__label">
        {label}
        <span className="mono insert-menu__attr">{attr}</span>
      </label>
      {children}
      <p className="insert-menu__hint">{hint}</p>
      {problem && (
        <p className="insert-menu__problem" role="alert">
          {problem}
        </p>
      )}
    </div>
  );
}

/** One of the template's own options. */
export function OptionField({
  option,
  value,
  onChange,
}: {
  option: TemplateOption;
  value: string;
  onChange: (next: string) => void;
}) {
  const hint =
    option.flag === NO_RESTORE
      ? "On by default here, unlike dotnet's: a restore is a build step, and what it writes into obj/ is not something a document should own."
      : option.description;
  return (
    <div className="insert-menu__field">
      <label className="insert-menu__label">
        {option.names.join(", ")}
        {option.default !== null && (
          <span className="mono insert-menu__attr">default {option.default}</span>
        )}
      </label>
      {option.kind === "bool" ? (
        <label className="new-project__check">
          <input
            type="checkbox"
            checked={(value || "false").toLowerCase() === "true"}
            onChange={(event) => onChange(event.target.checked ? "true" : "false")}
          />
          {option.description}
        </label>
      ) : option.kind === "choice" ? (
        <select
          className="insert-menu__input"
          value={value}
          onChange={(event) => onChange(event.target.value)}
        >
          {/* A choice may be left alone: no flag is written, and `dotnet`
              decides. Not the same as picking the value that happens to be
              the default today. */}
          <option value="">(leave to dotnet)</option>
          {option.choices.map((choice) => (
            <option key={choice.value} value={choice.value}>
              {choice.description
                ? `${choice.value} — ${choice.description}`
                : choice.value}
            </option>
          ))}
        </select>
      ) : (
        <input
          className="insert-menu__input"
          value={value}
          spellCheck={false}
          placeholder={option.default ?? ""}
          onChange={(event) => onChange(event.target.value)}
        />
      )}
      {option.kind !== "bool" && <p className="insert-menu__hint">{hint}</p>}
      {option.kind === "bool" && option.flag === NO_RESTORE && (
        <p className="insert-menu__hint">{hint}</p>
      )}
      {option.enabled_if && (
        <p className="insert-menu__hint">
          Applies when <span className="mono">{option.enabled_if}</span>.
        </p>
      )}
    </div>
  );
}


