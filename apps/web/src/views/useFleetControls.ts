import { useCallback, useEffect, useId, useRef, useState } from "react";
import { api } from "../api/client";
import type { FleetResponse } from "../api/types";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome, UiValue } from "../lib/uiActions";
export const FLEET_GRANTS = ["view", "edit", "execute"] as const;

export function useFleetControls(grantDescriptions: Record<string, string>) {
  const parent = `fleet:${useId()}`;
  const [fleet, setFleet] = useState<FleetResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [invitation, setInvitation] = useState<string | null>(null);
  const [pasted, setPasted] = useState("");
  const [dialled, setDialled] = useState("");
  const [hosting, setHosting] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [fallbackOpen, setFallbackOpen] = useState(false);
  const [receipt, setReceipt] = useState<UiValue>(null);
  const [hostResult, setHostResult] = useState<UiValue>(null);
  const pending = useRef(false), mounted = useRef(true), sequence = useRef(0);
  const hostRequest = useRef<string | null>(null);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; sequence.current++; }; }, []);
  const reload = useCallback(async () => {
    const ticket = ++sequence.current;
    try {
      const response = await api.fleet();
      if (mounted.current && sequence.current === ticket) { setFleet(response); setError(null); }
      return response;
    } catch (cause) {
      if (mounted.current && sequence.current === ticket) setError(cause instanceof Error ? cause.message : String(cause));
      return null;
    }
  }, []);
  useEffect(() => { void reload(); }, [reload]);
  const act = async (what: () => Promise<{ response: unknown; note?: string }>, signal?: AbortSignal): Promise<UiOutcome> => {
    if (pending.current || signal?.aborted || !mounted.current) return { status: "unavailable", reason: "Fleet controller is closed or busy" };
    pending.current = true; setBusy(true); setError(null);
    try {
      const result = await what();
      const value = { response: JSON.parse(JSON.stringify(result.response)) as UiValue,
        scope: "Actual local fleet API reply; observed-key writes use the roster lock; peer connectivity and noncooperating external writers are not verified" };
      if (mounted.current) { setReceipt(value); if (result.note) setNote(result.note); await reload(); }
      return { status: "completed", value };
    } catch (cause) {
      const reason = cause instanceof Error ? cause.message : String(cause);
      if (mounted.current) { setError(reason); setReceipt({ error: reason, effects: "Inspect the local roster before retrying" }); }
      return { status: "unavailable", reason };
    } finally { pending.current = false; if (mounted.current) setBusy(false); }
  };
  const phrase = () => act(async () => {
    if (hostRequest.current) throw Error("A pairing host request is already waiting; inspect its result or expiry");
    const response = await api.fleetPhrase();
    const job = crypto.randomUUID(); hostRequest.current = job;
    if (mounted.current) { setHosting(response.phrase); setHostResult({ job, status: "waiting", seconds: response.seconds }); }
    void api.fleetHost(response.phrase).then(answer => {
      if (!mounted.current || hostRequest.current !== job) return;
      setHosting(null); setHostResult({ job, status: "paired", response: JSON.parse(JSON.stringify(answer)) });
      setNote(`Paired with “${answer.machine.name}”. Compare the fingerprints: them ${answer.their_fingerprint}, you ${answer.our_fingerprint}.`);
      void reload();
    }, cause => {
      if (!mounted.current || hostRequest.current !== job) return;
      const reason = cause instanceof Error ? cause.message : String(cause);
      setHosting(null); setError(reason); setHostResult({ job, status: "failed", error: reason });
    }).finally(() => {
      if (hostRequest.current === job) {
        hostRequest.current = null;
        if (mounted.current) setHostResult(current => current && typeof current === "object" && !Array.isArray(current) ? { ...current, request_finished: true } : current);
      }
    });
    return { response, note: `Read this out and type it on the other machine. ${response.seconds} seconds.` };
  });
  const pair = () => act(async () => {
    const captured = dialled.trim(); if (!captured) throw Error("Enter a pairing phrase");
    const response = await api.fleetPair(captured);
    if (mounted.current) setDialled(current => current.trim() === captured ? "" : current);
    return { response, note: `Paired with “${response.machine.name}”. ${response.note} Them ${response.their_fingerprint}, you ${response.our_fingerprint}.` };
  });
  const invite = () => act(async () => {
    const response = await api.fleetInvite(); if (mounted.current) setInvitation(response.invitation);
    return { response, note: response.note };
  });
  const accept = () => act(async () => {
    const captured = pasted.trim(); if (!captured) throw Error("Enter a fleet invitation");
    const response = await api.fleetAccept(captured);
    if (mounted.current) setPasted(current => current.trim() === captured ? "" : current);
    return { response, note: `Added “${response.machine.name}”. It can view and edit; execute is off until you give it — and remember to accept this machine's invitation over there too, or it will be refused when it dials.` };
  });
  const copy = async (): Promise<UiOutcome> => {
    if (!invitation) return { status: "unavailable", reason: "Request this machine's invitation first" };
    try {
      if (!navigator.clipboard) throw Error("Clipboard is unavailable");
      await navigator.clipboard.writeText(invitation);
      if (mounted.current) setNote("Copied. Paste it on the other machine.");
      return { status: "completed", value: { copied: true } };
    } catch (cause) {
      const reason = `${String(cause)}. Read or select the displayed invitation directly; browser clipboard access may require a human gesture.`;
      if (mounted.current) setError(reason);
      return { status: "requires_input", request: `${parent}:clipboard`, reason };
    }
  };
  const revision = useUiRevision(() => [fleet, error, note, fallbackOpen, invitation, pasted, dialled, hosting, hostResult, pending.current, receipt]);
  const actions: { id: string; label: string; parameters?: UiCommand["parameters"]; enabled?: () => boolean; execute: UiCommand["execute"] }[] = [
    { id: "reload", label: "Refresh local fleet roster", execute: async () => ({ status: "completed", value: { refreshed: !!await reload() } }) },
    { id: "phrase", label: "Get fleet pairing phrase", enabled: () => !hostRequest.current, execute: phrase },
    { id: "dialled", label: "Set fleet pairing phrase", parameters: { text: { type: "string", maxLength: 8192 } }, execute: async args => { setDialled(args.text as string); return { status: "completed", value: { draft_updated: true } }; } },
    { id: "pair", label: "Pair fleet machine by phrase", enabled: () => !!dialled.trim(), execute: pair },
  ];
  if (fallbackOpen) actions.push(
    { id: "invite", label: "Show local fleet invitation", execute: invite },
    { id: "pasted", label: "Set fleet invitation draft", parameters: { text: { type: "string", maxLength: 65536 } }, execute: async args => { setPasted(args.text as string); return { status: "completed", value: { draft_updated: true } }; } },
    { id: "accept", label: "Accept fleet invitation", enabled: () => !!pasted.trim(), execute: accept },
    { id: "copy", label: "Copy local fleet invitation", enabled: () => !!invitation, execute: copy },
  );
  for (const machine of fleet?.machines ?? []) {
    for (const grant of FLEET_GRANTS) actions.push({ id: `grant:${machine.public_key}:${grant}`, label: `Set fleet grant ${machine.name} ${grant}`,
      parameters: { on: { type: "boolean" } }, execute: async args => act(async () => ({ response: await api.fleetGrant(machine.name, grant, args.on as boolean, machine.public_key) })) });
    actions.push({ id: `remove:${machine.public_key}`, label: `Remove fleet machine ${machine.name}`, execute: async () => act(async () => {
      const response = await api.fleetRemove(machine.name, machine.public_key); return { response, note: response.note };
    }) });
  }
  const invoke = useUiCommands(actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: action.parameters ?? {}, prerequisites: ["Existing local-only fleet API authority; peer routes do not expose fleet writes"],
    read: () => ({ revision: revision(), enabled: !pending.current && (action.enabled?.() ?? true) }), execute: action.execute,
  } })));
  const human = (id: string, args = {}) => void invoke(`${parent}:${id}`, args).catch(cause => { if (mounted.current) setError(String(cause)); });
  const metadata = { fleet: fleet ? { this_machine: fleet.this_machine,
    machines: fleet.machines.map(machine => ({ name: machine.name, kind: machine.kind, grants: machine.grants, public_key_prefix: `${machine.public_key.slice(0, 12)}…` })),
    reach: fleet.reach, reach_note: fleet.reach_note, note: fleet.note } : null,
    error, note, busy, dialled, hosting, grant_descriptions: grantDescriptions,
    host_request_active: !!hostRequest.current, raw_reply_available: !!receipt || !!hostResult,
    fallback_open: fallbackOpen, invitation: fallbackOpen ? invitation : null, invitation_draft: fallbackOpen ? pasted : null };
  useUiReading("local fleet state", metadata);
  return { fleet, error, note, invitation, pasted, dialled, hosting, busy, fallbackOpen, setFallbackOpen,
    setPasted: (text: string) => { if (!pending.current) setPasted(text); },
    setDialled: (text: string) => { if (!pending.current) setDialled(text); },
    human, copy, receipt, hostResult };
}
