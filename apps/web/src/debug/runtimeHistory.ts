import { observationText, type RuntimeRun } from "./runtimeObservations";
import type { WorkspaceStorage } from "../embed/storage";
import type { ObservationReading } from "./observations";

/** Reuses transactional workspace storage, with the same run wire model as Rust. */
export function runRepository(storage: WorkspaceStorage) {
  const path = (id: string) => {
    if (!/^[a-zA-Z0-9-]+$/.test(id)) throw Error("Invalid run identity");
    return `runs/${id}.json`;
  };
  const parse = (bytes: Uint8Array): RuntimeRun => {
    const run = JSON.parse(new TextDecoder().decode(bytes)) as RuntimeRun;
    if (run.schema_version !== 1 || !Array.isArray(run.events) || !Array.isArray(run.sources) || !run.sources.length) throw Error("Unsupported/corrupt runtime record");
    return run;
  };
  const verify = async (run: RuntimeRun) => {
    for (const source of run.sources) {
      const hash = await crypto.subtle.digest("SHA-256", new Uint8Array(source.bytes));
      if ([...new Uint8Array(hash)].map(b => b.toString(16).padStart(2, "0")).join("") !== source.revision) throw Error("Captured source digest mismatch");
    }
    if (new Set(run.events.map(event => event.id)).size !== run.events.length) throw Error("Duplicate event identity");
  };
  return {
    async list() {
      const runs = (await storage.snapshot()).filter(file => file.path.startsWith("runs/")).map(file => parse(file.bytes));
      for (const run of runs) await verify(run);
      return runs;
    },
    async save(run: RuntimeRun, pinUpdate = false) {
      await verify(run);
      const target = path(run.id);
      const previous = await storage.read(target);
      let candidate = run;
      if (previous) {
        const old = parse(previous.bytes);
        if (JSON.stringify(old.sources) !== JSON.stringify(run.sources) || old.started_unix_ms !== run.started_unix_ms ||
          JSON.stringify(run.events.slice(0, old.events.length)) !== JSON.stringify(old.events)) throw Error("Captured runtime evidence is immutable");
        if (old.status !== "running" && (run.status !== old.status || JSON.stringify(run.events) !== JSON.stringify(old.events))) throw Error("Finished run is immutable");
        if (old.truncated && !run.truncated) throw Error("Cannot erase truncation");
        candidate = { ...run, pinned: pinUpdate ? run.pinned : old.pinned };
      }
      const bytes = new TextEncoder().encode(JSON.stringify(candidate));
      if (bytes.length > 16 * 1024 * 1024) throw Error("Retained runtime record exceeds 16 MiB");
      await storage.write(target, bytes, previous?.revision ?? null);
      if (candidate.status !== "running") {
        const terminal = (await storage.snapshot()).filter(file => file.path.startsWith("runs/")).map(file => ({ file, run: parse(file.bytes) }))
          .filter(item => !item.run.pinned && item.run.status !== "running").sort((a,b) => a.run.started_unix_ms - b.run.started_unix_ms);
        let retainedBytes = terminal.reduce((sum, item) => sum + item.file.bytes.length, 0);
        let count = terminal.length;
        for (const item of terminal) if (count > 100 || retainedBytes > 256 * 1024 * 1024 || Date.now() - item.run.started_unix_ms > 30 * 86400000) {
          await storage.delete(item.file.path, item.file.revision); retainedBytes -= item.file.bytes.length; count--;
        }
      }
      return candidate;
    },
    async remove(id: string) { const old = await storage.read(path(id)); if (old) await storage.delete(old.path, old.revision); },
    close() { storage.close(); },
  };
}

export function runtimeReading(run: RuntimeRun, invocation?: string): ObservationReading {
  const source = run.sources[0];
  let providerVersion = "unknown";
  try { providerVersion = JSON.parse(run.detail ?? "{}").provider_version ?? "unknown"; } catch { /* Legacy plain-text detail. */ }
  const locations: ObservationReading["locations"] = [];
  const selected = run.events.filter(event => !invocation || (event.invocation ?? "unknown") === invocation);
  const annotations = selected.map(event => {
    let location: string | null = null;
    const anchor = event.observation.kind === "exception" ? event.observation.throwing ?? event.emission : event.emission;
    if (anchor?.source === source.id && anchor.bytes) {
      location = `${source.revision}:byte:${anchor.bytes.start}`;
      if (!locations.some(item => item.id === location)) locations.push({ id: location, revision: source.revision,
        line: new TextDecoder().decode(new Uint8Array(source.bytes.slice(0, anchor.bytes.start))).split("\n").length - 1,
        location: { source: source.id, bytes: anchor.bytes, precision: anchor.precision } });
    }
    return { evidence: event.id, provider: event.provider, payload_schema: "hick.runtime-event.v1", location,
      scope: { session: run.id, run: run.id, stop: "runtime", frame: null, invocation: event.invocation },
      payload: { kind: event.observation.kind, event, run_status: run.status, truncated: run.truncated } };
  });
  return { schema_version: 1, source, scope: { session: run.id, run: run.id, stop: "runtime", frame: null, invocation: invocation ?? null },
    locations, annotations, provider_annotations: selected.map(event => {
      const annotation = annotations.find(item => item.evidence === event.id)!;
      const location = locations.find(item => item.id === annotation.location) ?? null;
      return { evidence: event.id, provider: event.provider, provider_version: providerVersion,
        payload_schema: `hick.${event.observation.kind}.v1`,
        origin: { kind: "runtime" as const, session: null, run: run.id, stop: null, frame: null, invocation: event.invocation },
        location, source_precision: location?.location.precision ?? "unknown",
        measurement_semantics: event.observation.kind === "profile" ? event.observation.aggregation : null,
        uncertainty: event.observation.kind === "profile" ? "Profiler overhead and timing variability are not estimated" : null,
        payload: event.observation };
    }), unsupported: ["model semantic support", ...(run.truncated ? ["complete event coverage (truncated)"] : [])] };
}

export function runtimeText(run: RuntimeRun, invocation?: string) {
  return run.events.filter(event => !invocation || (event.invocation ?? "unknown") === invocation).map(event =>
    `${event.id} · invocation ${event.invocation ?? "unknown"} · ${event.emission?.precision ?? "unknown location"}\n` +
    observationText(event.observation)).join("")
    .replaceAll("\r", "\\r").replaceAll("\u001b", "\\x1b");
}
