import { useId } from "react";
import { api, ApiError, request } from "../api/client";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField, UiValue } from "../lib/uiActions";
import type { Terminals } from "./useTerminals";
const text = { type: "string" as const, maxLength: 8192 };
const json = (value: unknown): UiValue => JSON.parse(JSON.stringify(value));

export function useTerminalActions(controller: Terminals) {
  const instance = useId(); const parent = `terminals:${instance}`;
  const revision = useUiRevision(() => [controller.sessions, controller.anchors, controller.turbo, controller.error]);
  const commands: { key: string; command: UiCommand }[] = [];
  const done = (value: UiValue = { dispatch: "requested", result: "Read current terminal state/output" }) => ({ status: "completed" as const, value });
  function add(id: string, label: string, parameters: Record<string, UiField>, execute: UiCommand["execute"], value?: UiValue, subject = parent) {
    commands.push({ key: `${parent}/${id}`, command: { label, subject, parent, parameters, prerequisites: ["Local host terminal policy and actual session state apply; terminal commands may have side effects"], read: () => ({ revision: revision(), enabled: true, ...(value === undefined ? {} : { value }) }), execute } });
  }
  const accepted = (ok: boolean) => ok ? done({ host_request_completed: true, scope: "Actual host request reply; inspect current supervision/output for process and recording effects" })
    : { status: "unavailable" as const, reason: "Host terminal request failed or its controller closed; inspect the supervision error and actual state" };
  const state = json({ sessions: controller.sessions, attention: controller.attention, anchors: controller.anchors, turbo: controller.turbo, error: controller.error });
  add("state", "Read terminal supervision state", { offset: { type: "number", minimum: 0, maximum: 100000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify(state)); const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds terminal supervision state" };
    return done({ encoding: "JSON encoded as UTF-8 bytes", bytes: [...bytes.slice(offset, end)], span: [offset, end], next: end < bytes.length ? end : null, total_bytes: bytes.length });
  }, { sessions: controller.sessions.length, attention: controller.attention.length, turbo: controller.turbo });
  add("open", "Open host terminal", { cwd: text, argv: { type: "array", maxItems: 128, items: text }, title: text, monitor: { type: "boolean" }, worktree_branch: text }, async args => {
    const session = await controller.open({ cwd: String(args.cwd) || undefined, argv: args.argv as string[], title: String(args.title) || undefined, monitor: Boolean(args.monitor), worktree_branch: String(args.worktree_branch) || undefined });
    return session ? { status: "queued", job: session.id, value: json(session) } : { status: "unavailable", reason: "No terminal session reply was obtained; inspect host supervision before retrying" };
  });
  add("refresh", "Refresh terminal supervision", {}, async () => { controller.refresh(); return done(); });
  add("turbo", "Set terminal automatic-answer mode", { enabled: { type: "boolean" } }, async (args, actor) => {
    if (actor.kind === "agent" && args.enabled) return { status: "requires_input", reason: "A human must enable automatic answers in the terminal controls; observe turbo state afterward" };
    return accepted(await controller.setTurbo(Boolean(args.enabled)));
  });
  for (const session of controller.sessions) {
    const subject = `${parent}/${session.id}`;
    add(`${session.id}/output`, `Read terminal output: ${session.title}`, { view: { ...text, choices: ["screen", "replay"] }, offset: { type: "number", minimum: 0, maximum: 100_000_000, integer: true }, limit: { type: "number", minimum: 1, maximum: 16384, integer: true }, revision: text }, async args => done(await request<UiValue>("GET", `/api/terminals/${encodeURIComponent(session.id)}/output?${new URLSearchParams(Object.fromEntries(Object.entries(args).map(([key,value]) => [key,String(value)])))}`)), undefined, subject);
    add(`${session.id}/input`, `Input terminal bytes: ${session.title}`, { data: { type: "string", maxLength: 16384 } }, async (args, actor) => {
      if (actor.kind === "agent" && session.prompt) return { status: "requires_input", reason: "Terminal is asking for a decision. Use the human attention card; observe its actual result afterward" };
      try { await api.terminalInput(session.id, String(args.data), actor.kind === "agent"); }
      catch (error) { controller.refresh(); if (actor.kind === "agent" && error instanceof ApiError && error.status === 409) return { status: "requires_input", reason: error.message }; throw error; }
      controller.refresh(); return done();
    }, undefined, subject);
    add(`${session.id}/answer`, `Answer terminal prompt: ${session.title}`, { send: text }, async (args, actor) => {
      if (actor.kind === "agent") return { status: "requires_input", reason: "Terminal prompt authority is external/opaque. A human must answer the attention card; this request does not attest approval" };
      return accepted(await controller.answer(session.id, String(args.send)));
    }, { prompt_present: !!session.prompt, authority: "Actual human attention-card answer required" }, subject);
    add(`${session.id}/interrupt`, `Interrupt terminal: ${session.title}`, {}, async () => { return accepted(await controller.interrupt(session.id)); }, undefined, subject);
    add(`${session.id}/close`, `Close terminal: ${session.title}`, {}, async () => { return accepted(await controller.close(session.id)); }, undefined, subject);
    add(`${session.id}/anchor`, `Anchor terminal: ${session.title}`, { document: text, container: text }, async args => { return accepted(await controller.anchor(session.id, String(args.document), String(args.container))); }, undefined, subject);
    add(`${session.id}/unanchor`, `Unanchor terminal: ${session.title}`, {}, async () => { return accepted(await controller.unanchor(session.id)); }, undefined, subject);
    add(`${session.id}/resume`, `Resume terminal recording: ${session.title}`, {}, async () => { return accepted(await controller.resumeAnchor(session.id)); }, undefined, subject);
  }
  useUiCommands(commands);
}
