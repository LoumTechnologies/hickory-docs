import { useId } from "react";
import { MoreActions } from "../components/MoreActions";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
type Props = {
  name: string; document: boolean; readRevision: () => unknown;
  onOpen: (tool: "connections" | "execution" | "review" | "export") => void;
};
export function FileToolMenu(props: Props) {
  return <MoreActions label={`Tools for ${props.name}`}><FileToolButtons {...props} /></MoreActions>;
}
function FileToolButtons({ name, document, readRevision, onOpen }: Props) {
  const parent = `file-tools:${useId()}`;
  const revision = useUiRevision(() => [name, document, readRevision()]);
  const actions = [ { id: "connections" as const, label: "Connection display" }, { id: "execution" as const, label: "Tests and execution" },
    { id: "review" as const, label: "Review this file" }, ...(document ? [{ id: "export" as const, label: "Export document snapshot" }] : []) ];
  const invoke = useUiCommands(actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    label: action.label, parent, subject: parent, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }),
    execute: async () => { onOpen(action.id); return { status: "completed", value: { panel_requested: action.id, file: name } }; },
  } })));
  return <>{actions.map(action => <button key={action.id} type="button" onClick={() => { void invoke(`${parent}:${action.id}`); }}>{action.label}</button>)}</>;
}
