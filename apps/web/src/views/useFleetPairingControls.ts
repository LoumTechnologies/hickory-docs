import { useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import { pairingPending, type FleetPairingJob } from "../api/fleetPairings";
import { useUiCommands, useUiRevision, useUiScope } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome, UiValue } from "../lib/uiActions";

const value = (job: FleetPairingJob): UiValue => JSON.parse(JSON.stringify(job));
type Attempt = { id: string; mode: "host" | "dial"; phrase: string; cancel: boolean };
/** A transport failure keeps the exact retry identity; it never invents nonexecution. */
export function useFleetPairingControls(parent: string, onPaired: () => void) {
  const scope = useUiScope();
  const owner = scope?.window ?? null;
  const [jobs, setJobs] = useState<FleetPairingJob[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [uncertain, setUncertain] = useState(false);
  const epoch = useRef(0);
  const lock = useRef(false), attempt = useRef<Attempt | null>(null);
  const latest = useRef(jobs); latest.current = jobs;
  const lifetime = useRef<{ owner: string | null; live: boolean }>({ owner, live: true });
  const paired = useRef(onPaired); paired.current = onPaired;
  const adopt = (job: FleetPairingJob) => {
    const previous = latest.current.find(item => item.id === job.id);
    if (previous && !pairingPending(previous) && pairingPending(job)) return;
    if (previous?.cancel_requested) job = { ...job, cancel_requested: true };
    const next = [...latest.current.filter(item => item.id !== job.id), job];
    latest.current = next; setJobs(next);
    if (job.status === "paired" && previous?.status !== "paired") paired.current();
  };
  useEffect(() => {
    const life = { owner, live: true }; lifetime.current = life;
    latest.current = []; setJobs([]); attempt.current = null; lock.current = false;
    setPending(false); setUncertain(false); setError(null);
    const refresh = async () => {
      if (!owner) return;
      try {
        const response = await api.fleetPairings(owner);
        if (!life.live) return;
        response.jobs.filter(job => !latest.current.some(existing => existing.id === job.id)).forEach(adopt); setError(null);
      } catch (cause) { if (life.live) setError(String(cause)); }
    };
    void refresh();
    return () => {
      life.live = false; epoch.current++;
      if (attempt.current) attempt.current.cancel = true;
      if (owner) for (const job of latest.current.filter(pairingPending)) void api.cancelFleetPairing(owner, job.id).catch(() => {});
    };
  }, [owner]);
  useEffect(() => {
    const life = lifetime.current;
    const active = jobs.filter(pairingPending);
    if (!owner || !active.length) return;
    const timer = setTimeout(async () => {
      const version = epoch.current;
      try {
        for (const job of active) {
          const response = await api.fleetPairing(owner, job.id);
          if (!life.live || epoch.current !== version) return;
          adopt(response);
        }
      } catch (cause) { if (life.live) setError(String(cause)); }
    }, 800);
    return () => clearTimeout(timer);
  }, [owner, jobs]);
  const start = async (mode: "host" | "dial", phrase: string, signal?: AbortSignal): Promise<UiOutcome> => {
    const life = lifetime.current;
    if (signal?.aborted) return { status: "unavailable", reason: "Pairing start cancelled before submission" };
    if (!owner || life.owner !== owner || !life.live) return { status: "unavailable", reason: "Connect this UI window before pairing; no legacy fallback is used" };
    if (lock.current || latest.current.some(pairingPending)) return { status: "unavailable", reason: "Inspect or cancel the active pairing request first" };
    let captured = attempt.current;
    if (captured && (captured.mode !== mode || captured.phrase !== phrase)) return { status: "requires_input", request: `${parent}:pairing-retry`, reason: "The previous start outcome is unknown. Retry the same phrase or refresh and inspect its request before changing arguments" };
    if (!captured) { captured = { id: crypto.randomUUID(), mode, phrase, cancel: false }; attempt.current = captured; }
    const cancelStart = () => { captured!.cancel = true; };
    signal?.addEventListener("abort", cancelStart, { once: true });
    lock.current = true; epoch.current++; setPending(true);
    try {
      const response = await api.startFleetPairing(owner, captured.id, mode, phrase);
      if (!life.live || captured.cancel) {
        if (pairingPending(response.job)) void api.cancelFleetPairing(owner, response.job.id).catch(() => {});
      }
      if (life.live) { adopt(response.job); setError(null); setUncertain(false); attempt.current = null; }
      return pairingPending(response.job) ? { status: response.job.status as "queued" | "running", job: response.job.id, value: value(response.job) } : { status: "completed", value: value(response.job) };
    } catch (cause) {
      const status = (cause as { status?: number })?.status;
      const refused = typeof status === "number" && status >= 400 && status < 500;
      if (life.live) {
        if (refused) attempt.current = null;
        setError(refused ? String(cause) : `${String(cause)}. Start outcome unknown; retry uses the same request ID.`); setUncertain(!refused);
      }
      return { status: "unavailable", reason: `${String(cause)}; start outcome unknown, inspect before changing the request` };
    } finally { signal?.removeEventListener("abort", cancelStart); if (life.live) { lock.current = false; setPending(false); } }
  };
  const refresh = async (): Promise<UiOutcome> => {
    const life = lifetime.current;
    if (!owner || !life.live) return { status: "unavailable", reason: "Originating window is unavailable" };
    try {
      const version = ++epoch.current;
      const response = await api.fleetPairings(owner);
      if (!life.live || version !== epoch.current) return { status: "unavailable", reason: "Originating window closed" };
      const previouslyPaired = new Set(latest.current.filter(job => job.status === "paired").map(job => job.id));
      latest.current = response.jobs; setJobs(response.jobs);
      if (response.jobs.some(job => job.status === "paired" && !previouslyPaired.has(job.id))) paired.current();
      const found = response.jobs.find(job => job.request_id === attempt.current?.id);
      if (found) { attempt.current = null; setUncertain(false); }
      setError(null);
      return { status: "completed", value: { refreshed: true, requests: response.jobs.length } };
    } catch (cause) { setError(String(cause)); return { status: "unavailable", reason: String(cause) }; }
  };
  const revision = useUiRevision(() => [jobs, pending, uncertain, error]);
  const commands = [{ id: "refresh", label: "Refresh fleet pairing requests", enabled: !pending, execute: refresh },
    { id: "cancel-start", label: "Request cancellation of pending fleet pairing start", enabled: pending || uncertain, execute: async (): Promise<UiOutcome> => {
      if (!attempt.current) return { status: "unavailable", reason: "No pending pairing start" };
      attempt.current.cancel = true;
      return { status: "completed", value: { cancellation_requested: true, scope: "Forwarded when the host job identity is recovered; this does not confirm stopped work", rollback: false } };
    } }];
  for (const job of jobs) {
    commands.push({ id: `cancel:${job.id}`, label: `Cancel fleet pairing ${job.id}`, enabled: pairingPending(job), execute: async () => {
      const life = lifetime.current;
      try {
        const version = ++epoch.current;
        const response = await api.cancelFleetPairing(owner!, job.id);
        if (life.live && version === epoch.current) adopt(response.job);
        return { status: "completed", value: { cancellation_requested: true, job: value(response.job), rollback: false } };
      } catch (cause) { return { status: "unavailable", reason: String(cause) }; }
    } });
    commands.push({ id: `forget:${job.id}`, label: `Forget fleet pairing result ${job.id}`, enabled: !pairingPending(job), execute: async () => {
      const life = lifetime.current;
      try {
        const version = ++epoch.current;
        const response = await api.forgetFleetPairing(owner!, job.id);
        if (life.live && version === epoch.current) { latest.current = latest.current.filter(item => item.id !== job.id); setJobs(latest.current); }
        return { status: "completed", value: { forgotten: response.forgotten, rollback: false } };
      } catch (cause) { return { status: "unavailable", reason: String(cause) }; }
    } });
  }
  const invoke = useUiCommands(commands.map(command => ({ key: `${parent}:pairing:${command.id}`, command: {
    parent, subject: parent, label: command.label, parameters: {}, prerequisites: ["Actual requests owned by this local UI window"],
    read: () => ({ revision: revision(), enabled: !!owner && command.enabled }), execute: command.execute,
  } })));
  useUiReading("Fleet pairing request status", { connected: !!owner, pending, uncertain_start: uncertain, error,
    jobs: jobs.map(job => ({ id: job.id, mode: job.mode, status: job.status, phase: job.phase, cancel_requested: job.cancel_requested })),
    scope: "Last host-observed in-memory requests for this window. Cancellation does not roll back enrollment. Full replies require disclosure." });
  return { start, jobs, pending, uncertain, error, active: jobs.some(pairingPending), human: (id: string) => void invoke(`${parent}:pairing:${id}`).catch(cause => setError(String(cause))) };
}
