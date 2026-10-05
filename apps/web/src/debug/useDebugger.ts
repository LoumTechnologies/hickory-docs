// One debug session per open document, and the editor state that follows it.
//
// The hook owns three things the UI reads and the editor draws: where
// execution is paused, what is in scope there, and which breakpoints the
// adapter could actually bind. Everything arrives as an event, because a
// `continue` takes as long as the program does.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getWorkspaceRealtime, type Realtime } from "../api/realtime";
import type { TranscriptEvent } from "../api/types";
import { DebugClient } from "./client";
import type {
  BreakpointStatus,
  DebugBreakpoint,
  DebugCapabilities,
  DebugEvent,
  Frame,
  Step,
  Variable,
} from "./client";

export type DebugStatus = "idle" | "starting" | "paused" | "running" | "finished" | "failed";

/** One watched expression, and what it held the last time anyone asked. */
export interface Watch {
  expression: string;
  /** Null until the first answer arrives, and again after the program ends. */
  value: string | null;
}

export interface DebugSession {
  /** A tool this machine can fetch, when a missing one is why the last
   * attempt failed. Null the rest of the time. */
  offerInstall: { kind: string; language: string } | null;
  status: DebugStatus;
  message: string | null;
  /** The generated file being debugged, when one was named. */
  program: string | null;
  capabilities: DebugCapabilities | null;
  /** 0-based document line execution is paused on. */
  pausedLine: number | null;
  frames: Frame[];
  variables: Variable[];
  selectedFrame: number | null;
  /** Where the user has asked to stop, whether or not it bound. */
  breakpoints: HeldBreakpoint[];
  /** How the program ended, once it has: its exit code, when reported. */
  exitCode: number | null;
  /**
   * The build that ran before there was a program to launch, when one did.
   *
   * Empty for Python, Node and Go, where the generated file IS the program.
   * Kept whether the start succeeded or failed — the failing case is the one
   * it exists for, because a build that fails says why in the compiler's own
   * words and "build failed" throws all of that away.
   */
  buildOutput: TranscriptEvent[];
  lastValue: { expression: string; value: string; type: string | null } | null;
  /** Expressions re-evaluated on every pause and frame change. */
  watches: Watch[];
  /**
   * The adapter's exception filters that are switched on.
   *
   * Ids, not names — `capabilities.exception_filters` carries the labels a
   * person reads. Kept across runs like breakpoints are: "stop on uncaught
   * exceptions" is a standing preference, not a property of one session.
   */
  exceptionFilters: string[];

  /** Debug one generated file. Omitted means the first debuggable one. */
  start(program?: string): void;
  stop(): void;
  step(how: Step): void;
  jumpTo(line: number): void;
  runTo(line: number): void;
  toggleBreakpoint(line: number): void;
  /** Turn one of the adapter's exception filters on or off. */
  toggleExceptionFilter(id: string): void;
  /**
   * Attach a condition, a hit count, or a log message to the breakpoint on
   * this line — or clear them, by passing empty strings.
   *
   * All three have been carried by `hick-dap::Breakpoint` and forwarded to
   * the adapter since the debugger was written; until this existed nothing
   * could set one, and the gutter's own `cm-bp-conditional` styling was
   * unreachable.
   */
  setBreakpointCondition(
    line: number,
    patch: { condition?: string; hit_condition?: string; log_message?: string },
  ): void;
  selectFrame(id: number): void;
  evaluate(expression: string, context?: "hover" | "watch" | "repl"): void;
  /** Ask for a value and get it back, for the hover tooltip. */
  valueAt(expression: string): Promise<string | null>;
  /**
   * Ask for a value the way a person typed it: any expression, no
   * known-name guard, answered as a promise for the inline eval widget.
   */
  query(expression: string): Promise<string | null>;
  addWatch(expression: string): void;
  removeWatch(expression: string): void;
}

/**
 * A breakpoint as this pane holds it: what the adapter said about it, plus
 * the three things only the person knows.
 *
 * The adapter never reports a condition back, so it cannot come from
 * `BreakpointStatus` — it has to survive every status the server sends.
 */
export interface HeldBreakpoint extends BreakpointStatus {
  /** Stop only when this is true, in the debuggee's own language. */
  condition?: string;
  /** Stop only on the Nth hit — the adapter's own syntax. */
  hit_condition?: string;
  /** Log this and continue instead of stopping. */
  log_message?: string;
}

/** Whether a breakpoint carries anything beyond "stop here". */
export function isConditional(breakpoint: HeldBreakpoint): boolean {
  return Boolean(breakpoint.condition || breakpoint.hit_condition || breakpoint.log_message);
}

/**
 * Put each breakpoint where the adapter actually bound it, keeping what the
 * person attached to it.
 *
 * A dot drawn on the line you asked for, while the program stops on the line
 * below, is a gutter that disagrees with the debugger — and the debugger is
 * the one that is right. But the move must carry the condition with it: a
 * breakpoint that slides one line down and silently becomes unconditional
 * would stop on every pass, which is the opposite of what was asked for.
 * That is why the match is made against the status BEFORE the move is
 * applied, where both the requested and the bound line are still in hand.
 */
function followMoves(previous: HeldBreakpoint[], statuses: BreakpointStatus[]): HeldBreakpoint[] {
  return statuses.map((status) => {
    const held =
      previous.find((breakpoint) => breakpoint.line === status.line) ??
      (status.moved_to === undefined
        ? undefined
        : previous.find((breakpoint) => breakpoint.line === status.moved_to));
    const moved =
      status.moved_to === undefined || status.moved_to === status.line
        ? status
        : { ...status, line: status.moved_to, moved_to: undefined };
    return {
      ...moved,
      condition: held?.condition,
      hit_condition: held?.hit_condition,
      log_message: held?.log_message,
    };
  });
}

/**
 * The wire form: only what was actually set, so an adapter is never handed
 * an empty condition string to parse.
 */
function onTheWire(breakpoints: HeldBreakpoint[]): DebugBreakpoint[] {
  return breakpoints.map((breakpoint) => ({
    line: breakpoint.line,
    ...(breakpoint.condition ? { condition: breakpoint.condition } : {}),
    ...(breakpoint.hit_condition ? { hit_condition: breakpoint.hit_condition } : {}),
    ...(breakpoint.log_message ? { log_message: breakpoint.log_message } : {}),
  }));
}

/**
 * Whether an event from the channel is about THIS session.
 *
 * The workspace socket is shared by every plain-file pane, and a pane that
 * took a neighbour's `stopped` would show itself paused on a line of a
 * program it never started. A session-bearing event is ours when the
 * session is the one we hold; the ones from before there is a session —
 * `started`, a `build`, a `failed` start — name the file instead, and are
 * ours when they name ours. An event with neither (an older server, a
 * test's bare event) is taken, so a document's own socket keeps working
 * unchanged.
 */
export function eventIsOurs(
  event: DebugEvent,
  uri: string,
  session: string | null,
): boolean {
  if (event.event === "started" || event.event === "build") {
    return event.doc === undefined || event.doc === uri;
  }
  if (event.event === "failed") {
    if (event.session) return event.session === session;
    return event.doc === undefined || event.doc === uri;
  }
  return session !== null && event.session === session;
}

/** A session that does nothing: what a pane shows until its host has mounted. */
export const IDLE_SESSION: DebugSession = {
  offerInstall: null,
  status: "idle",
  message: null,
  program: null,
  capabilities: null,
  pausedLine: null,
  frames: [],
  variables: [],
  selectedFrame: null,
  breakpoints: [],
  exitCode: null,
  buildOutput: [],
  lastValue: null,
  watches: [],
  exceptionFilters: [],
  start() {},
  stop() {},
  step() {},
  jumpTo() {},
  runTo() {},
  toggleBreakpoint() {},
  toggleExceptionFilter() {},
  setBreakpointCondition() {},
  selectFrame() {},
  evaluate() {},
  valueAt: () => Promise.resolve(null),
  query: () => Promise.resolve(null),
  addWatch() {},
  removeWatch() {},
};

/** A debugger for one document, over the document's own socket. */
export function useDebugger(realtime: Realtime, docPath: string): DebugSession {
  const client = useMemo(() => {
    const channel = realtime.debug?.();
    return channel ? new DebugClient(channel) : null;
  }, [realtime]);

  // The way back. A client that can send but is never handed the socket's
  // inbound frames sits at "starting…" forever while the engine answers into
  // nothing, so this is registered as soon as the client exists.
  useEffect(() => {
    if (!client || !realtime.onDebugFrame) return;
    realtime.onDebugFrame((frame) => client.handleFrame(frame));
    return () => realtime.onDebugFrame?.(() => false);
  }, [client, realtime]);

  return useDebuggerOver(client, docPath);
}

// The workspace's one debug client, shared by every plain file.
//
// One, not one per pane, for the same reason the language client is one:
// the socket hands inbound frames to a single handler, and two clients on
// one connection would each take the other's answers. Each pane's hook
// filters the shared stream down to its own session (`eventIsOurs`).
let workspaceClient: DebugClient | null = null;
function workspaceDebugClient(): DebugClient | null {
  if (workspaceClient) return workspaceClient;
  const realtime = getWorkspaceRealtime();
  const channel = realtime?.debug?.();
  if (!realtime || !channel) return null;
  const client = new DebugClient(channel);
  realtime.onDebugFrame?.((frame) => client.handleFrame(frame));
  workspaceClient = client;
  return client;
}

/** Test seam: forget the workspace debug client. */
export function resetWorkspaceDebugger(): void {
  workspaceClient = null;
}

/**
 * A debugger for a plain file — `src/main.rs`, `app.py` — over the
 * workspace connection. The same verbs, the same events, at the file's
 * own path; the server debugs a file that is not a document as itself.
 */
export function useWorkspaceDebugger(path: string): DebugSession {
  const client = useMemo(() => workspaceDebugClient(), []);
  return useDebuggerOver(client, path);
}

export function useDebuggerOver(client: DebugClient | null, docPath: string, initialBreakpoints: readonly number[] = []): DebugSession {
  const [status, setStatus] = useState<DebugStatus>("idle");
  const [message, setMessage] = useState<string | null>(null);
  // A missing tool this machine can fetch, carried out of the failure so the
  // strip can offer a button instead of a command to go and type somewhere
  // else.
  //
  // It belongs to `message` and is cleared with it — `clearFailure` exists so
  // the two cannot come apart. They did once: installing the adapter started
  // the session successfully and left "No Python debugger on this machine"
  // sitting beside "finished — exit code 0".
  const [offerInstall, setOfferInstall] = useState<{
    kind: string;
    language: string;
  } | null>(null);
  const clearFailure = useCallback(() => {
    setMessage(null);
    setOfferInstall(null);
  }, []);
  const [capabilities, setCapabilities] = useState<DebugCapabilities | null>(null);
  const [pausedLine, setPausedLine] = useState<number | null>(null);
  const [frames, setFrames] = useState<Frame[]>([]);
  const [variables, setVariables] = useState<Variable[]>([]);
  const [selectedFrame, setSelectedFrame] = useState<number | null>(null);
  const [breakpoints, setBreakpoints] = useState<HeldBreakpoint[]>(() => initialBreakpoints.map((line) => ({ line, state: "bound" })));
  // Which file this session is running, for the panel to say so: "paused" is
  // ambiguous in a document that generates three programs.
  const [program, setProgram] = useState<string | null>(null);
  const [lastValue, setLastValue] =
    useState<{ expression: string; value: string; type: string | null } | null>(null);
  const [watches, setWatches] = useState<Watch[]>([]);
  // Kept across runs, like breakpoints: "stop on uncaught exceptions" is a
  // standing preference, not a property of one session.
  const [exceptionFilters, setExceptionFilters] = useState<string[]>([]);
  // Read from the `started` handler, which is not re-created per render.
  const exceptionFiltersRef = useRef<string[]>([]);
  exceptionFiltersRef.current = exceptionFilters;
  const [exitCode, setExitCode] = useState<number | null>(null);
  const [buildOutput, setBuildOutput] = useState<TranscriptEvent[]>([]);

  const sessionRef = useRef<string | null>(null);
  // The status as of right now, for callbacks that must not close over a
  // stale one — a breakpoint toggled a moment after the program ended would
  // otherwise still be sent to a session that cannot answer.
  const statusRef = useRef<DebugStatus>("idle");
  // Hover asks for a value and wants it back; the channel answers with an
  // event, so the request is parked here until its answer arrives.
  const pendingValues = useRef(new Map<string, (value: string | null) => void>());

  statusRef.current = status;

  const uri = useMemo(() => `hick:///${docPath.replace(/^\/+/, "")}`, [docPath]);

  useEffect(() => {
    if (!client) return;
    const off = client.on((event: DebugEvent) => {
      // A shared channel carries every pane's sessions; only ours is ours.
      // Before `started` there is no session to match, so a start that is
      // still in flight is recognised by the file it named.
      if (!eventIsOurs(event, uri, sessionRef.current)) return;
      switch (event.event) {
        case "build":
          // Replaces rather than appends: a build belongs to the start that
          // ran it, and the last one is the one on screen.
          setBuildOutput(event.events);
          break;
        case "started": {
          sessionRef.current = event.session;
          setCapabilities(event.capabilities);
          setBreakpoints((current) => followMoves(current, event.breakpoints));
          setStatus("running");
          clearFailure();
          setExitCode(null);
          // Re-applied to the new session. A standing "stop on uncaught"
          // that quietly stopped applying on the second run would be worse
          // than never having offered it.
          const standing = exceptionFiltersRef.current;
          if (client && standing.length > 0) {
            client.setExceptionBreakpoints(event.session, standing);
          }
          break;
        }
        case "stopped":
          setStatus("paused");
          setPausedLine(event.line);
          setFrames(event.frames);
          setVariables(event.variables);
          setSelectedFrame(event.selected_frame ?? event.frames[0]?.id ?? null);
          clearFailure();
          break;
        case "breakpoints":
          setBreakpoints((current) => followMoves(current, event.breakpoints));
          break;
        case "value": {
          setLastValue({ expression: event.expression, value: event.value, type: event.type });
          // Answers carry the expression they answer, and the watch list is
          // keyed by expression — that string IS the correlation id. A watch
          // and a hover for the same expression both get the answer, which
          // is fine: it is the same answer.
          setWatches((current) =>
            current.map((watch) =>
              watch.expression === event.expression ? { ...watch, value: event.value } : watch,
            ),
          );
          const waiting = pendingValues.current.get(event.expression);
          if (waiting) {
            waiting(event.value);
            pendingValues.current.delete(event.expression);
          }
          break;
        }
        case "finished":
          // The server reaped the session with the program: the id no longer
          // answers, so forget it NOW. A step, hover or stop sent after this
          // would be a request to a dead session, answered only by an error.
          sessionRef.current = null;
          setStatus("finished");
          setExitCode(event.exit_code ?? null);
          setPausedLine(null);
          setFrames([]);
          setVariables([]);
          setSelectedFrame(null);
          // The expressions survive — they are for the next run — but their
          // values belonged to a process that no longer exists.
          setWatches((current) => current.map((watch) => ({ ...watch, value: null })));
          break;
        case "ended":
          sessionRef.current = null;
          setStatus("idle");
          clearFailure();
          setPausedLine(null);
          setFrames([]);
          setVariables([]);
          setCapabilities(null);
          setWatches((current) => current.map((watch) => ({ ...watch, value: null })));
          break;
        case "failed": {
          const stillWaiting = [...pendingValues.current.values()];
          pendingValues.current.clear();
          for (const resolve of stillWaiting) resolve(null);
          // A breakpoint that could not be set is a fact about that
          // breakpoint. It is marked broken, with the reason on hover, and
          // says nothing in the panel — where it would outlive its cause and
          // read as if it were about whatever you did next.
          if (event.about === "breakpoints" || event.about === "start") {
            const broken = new Set(event.lines ?? []);
            if (broken.size > 0) {
              setBreakpoints((current) =>
                current.map((breakpoint) =>
                  broken.has(breakpoint.line)
                    ? { ...breakpoint, state: "refused" as const, message: event.message }
                    : breakpoint,
                ),
              );
              if (event.about === "breakpoints") break;
            }
          }
          // A hover that could not be answered is not news. It resolves to
          // nothing and the tooltip shows the type alone; putting the
          // adapter's `NameError` in the panel would report our own question
          // back to the person as if their program were wrong.
          if (stillWaiting.length === 0) {
            setMessage(event.message);
            setOfferInstall(event.offer_install ?? null);
          }
          // A failed STEP leaves the program where it was — still paused —
          // so only a failure with no session at all is fatal to the UI.
          // "Finished" already IS a terminal state: a stray failure arriving
          // after the program ended must not redress a clean end as a broken
          // one.
          if (!sessionRef.current && statusRef.current !== "finished") setStatus("failed");
          break;
        }
      }
    });
    return off;
  }, [client, uri]);

  // A window that closes mid-session leaves a process running; the server
  // sweeps it, but saying so promptly is cheaper than waiting for the sweep.
  useEffect(() => {
    return () => {
      if (client && sessionRef.current) client.stop(sessionRef.current);
    };
  }, [client]);

  const toggleBreakpoint = useCallback(
    (line: number) => {
      setBreakpoints((current) => {
        const without = current.filter((breakpoint) => breakpoint.line !== line);
        const next =
          without.length === current.length
            ? // Optimistically bound: the adapter's answer replaces this a
              // moment later, and a dot that appears only after a round trip
              // feels broken. Not "pending", which would flash a half-filled
              // dot on every click in a language that binds instantly.
              [...current, { line, state: "bound" as const }]
            : without;
        // Only a session that is still running can be told. After the
        // program ends the adapter answers `setBreakpoints` with "Server
        // disconnected unexpectedly", which is true and useless: the
        // breakpoint is for the NEXT run, and it is kept here until then.
        const session = sessionRef.current;
        if (client && session && (statusRef.current === "paused" || statusRef.current === "running")) {
          clearFailure();
          client.setBreakpoints(session, onTheWire(next));
        }
        return next.sort((a, b) => a.line - b.line);
      });
    },
    [client],
  );

  const toggleExceptionFilter = useCallback(
    (id: string) => {
      setExceptionFilters((current) => {
        const next = current.includes(id)
          ? current.filter((one) => one !== id)
          : [...current, id];
        const session = sessionRef.current;
        if (
          client &&
          session &&
          (statusRef.current === "paused" || statusRef.current === "running")
        ) {
          client.setExceptionBreakpoints(session, next);
        }
        return next;
      });
    },
    [client],
  );

  const setBreakpointCondition = useCallback(
    (
      line: number,
      patch: { condition?: string; hit_condition?: string; log_message?: string },
    ) => {
      setBreakpoints((current) => {
        // An empty string clears; `undefined` in the patch leaves that field
        // alone, so a popover can send only what it edited. Trimmed here
        // rather than in the caller: a condition of three spaces is not a
        // condition, and an adapter handed one reports a parse error about
        // code the person never wrote.
        const clean = (value: string | undefined) => {
          const trimmed = value?.trim();
          return trimmed ? trimmed : undefined;
        };
        const next = current.map((breakpoint) =>
          breakpoint.line === line
            ? {
                ...breakpoint,
                condition:
                  "condition" in patch ? clean(patch.condition) : breakpoint.condition,
                hit_condition:
                  "hit_condition" in patch
                    ? clean(patch.hit_condition)
                    : breakpoint.hit_condition,
                log_message:
                  "log_message" in patch ? clean(patch.log_message) : breakpoint.log_message,
              }
            : breakpoint,
        );
        // Same rule as toggling: only a live session can be told, and the
        // condition is kept here for the next run either way.
        const session = sessionRef.current;
        if (
          client &&
          session &&
          (statusRef.current === "paused" || statusRef.current === "running")
        ) {
          clearFailure();
          client.setBreakpoints(session, onTheWire(next));
        }
        return next;
      });
    },
    [client, clearFailure],
  );

  const start = useCallback(
    (program?: string) => {
      if (!client) return;
      setStatus("starting");
      clearFailure();
      setProgram(program ?? null);
      client.start(uri, onTheWire(breakpoints), program);
    },
    [client, uri, breakpoints, clearFailure],
  );

  const stop = useCallback(() => {
    if (client && statusRef.current === "starting") client.cancelStart(uri);
    if (client && sessionRef.current) {
      client.stop(sessionRef.current);
      return;
    }
    // No session to tell — the server already reaped it when the program
    // finished (or the start failed). Stop is then only a dismissal: clear
    // the strip's chrome locally, sending nothing to a session that is gone.
    setStatus("idle");
    clearFailure();
    setCapabilities(null);
    setExitCode(null);
  }, [client, uri]);

  const step = useCallback(
    (how: Step) => {
      if (!client || !sessionRef.current) return;
      setStatus("running");
      client.step(sessionRef.current, how, selectedFrame ?? undefined);
    },
    [client, selectedFrame],
  );

  const jumpTo = useCallback(
    (line: number) => {
      if (!client || !sessionRef.current) return;
      setStatus("running");
      client.jump(sessionRef.current, line);
    },
    [client],
  );

  const runTo = useCallback(
    (line: number) => {
      if (!client || !sessionRef.current) return;
      setStatus("running");
      client.runTo(sessionRef.current, line);
    },
    [client],
  );

  const evaluate = useCallback(
    (expression: string, context: "hover" | "watch" | "repl" = "repl") => {
      if (!client || !sessionRef.current) return;
      client.evaluate(sessionRef.current, expression, context, selectedFrame ?? undefined);
    },
    [client, selectedFrame],
  );

  const valueAt = useCallback(
    (expression: string): Promise<string | null> => {
      if (!client || !sessionRef.current || status !== "paused") return Promise.resolve(null);
      // Only names this frame has. The debugger is not a spell-checker: asking
      // it about every word the pointer crosses — a word in the prose, a tag
      // name, a comment — earns a `NameError` per hover, and the error is
      // about our question rather than about the program. The frame's own
      // variables are the static analysis that matters here, and they are
      // already in hand.
      const root = expression.split(/[.[]/)[0];
      if (!variables.some((variable) => variable.name === root)) {
        return Promise.resolve(null);
      }
      return new Promise((resolve) => {
        pendingValues.current.set(expression, resolve);
        client.evaluate(sessionRef.current!, expression, "hover", selectedFrame ?? undefined);
        // A hover that never answers must not leave the tooltip waiting
        // forever on a program that has moved on.
        setTimeout(() => {
          if (pendingValues.current.delete(expression)) resolve(null);
        }, 2000);
      });
    },
    [client, status, selectedFrame, variables],
  );

  const query = useCallback(
    (expression: string): Promise<string | null> => {
      if (!client || !sessionRef.current || statusRef.current !== "paused") {
        return Promise.resolve(null);
      }
      return new Promise((resolve) => {
        pendingValues.current.set(expression, resolve);
        // `repl` context: the person typed this, so a side effect is their
        // business — unlike a hover, which must never have one.
        client.evaluate(sessionRef.current!, expression, "repl", selectedFrame ?? undefined);
        setTimeout(() => {
          if (pendingValues.current.delete(expression)) resolve(null);
        }, 5000);
      });
    },
    [client, selectedFrame],
  );

  const addWatch = useCallback(
    (expression: string) => {
      const trimmed = expression.trim();
      if (!trimmed) return;
      setWatches((current) =>
        current.some((watch) => watch.expression === trimmed)
          ? current
          : [...current, { expression: trimmed, value: null }],
      );
      // Answer immediately when there is a frame to ask; otherwise the
      // re-evaluate effect covers it at the next pause.
      if (client && sessionRef.current && statusRef.current === "paused") {
        client.evaluate(sessionRef.current, trimmed, "watch", selectedFrame ?? undefined);
      }
    },
    [client, selectedFrame],
  );

  const removeWatch = useCallback((expression: string) => {
    setWatches((current) => current.filter((watch) => watch.expression !== expression));
  }, []);

  // Every pause and every frame change re-asks every watch: a watch showing
  // the value from two steps ago is worse than no watch at all. Keyed on the
  // joined expressions rather than the array so an answer arriving (which
  // replaces the array) does not re-ask the question it answers.
  const watchKey = watches.map((watch) => watch.expression).join("\n");
  useEffect(() => {
    if (status !== "paused" || !client || !sessionRef.current) return;
    for (const expression of watchKey ? watchKey.split("\n") : []) {
      client.evaluate(sessionRef.current, expression, "watch", selectedFrame ?? undefined);
    }
  }, [status, selectedFrame, watchKey, client]);

  const selectFrame = useCallback(
    (id: number) => {
      setSelectedFrame(id);
      const frame = frames.find((candidate) => candidate.id === id);
      // Selecting a frame moves the paused marker with it, because the line
      // you are looking at should be the line whose values you are reading.
      if (frame?.line !== undefined) setPausedLine(frame.line);
      if (capabilities?.frame_locals && client && sessionRef.current) client.state(sessionRef.current, id);
    },
    [frames, capabilities, client],
  );

  // One object per distinct state, not one per render: the document session
  // carries this as a field and is published by identity, so a fresh object
  // on every render would re-publish the session — and re-render the whole
  // workspace — on every keystroke of the document beside it.
  return useMemo(
    () => ({
      status,
      message,
      offerInstall,
      program,
      capabilities,
      pausedLine,
      frames,
      variables,
      selectedFrame,
      breakpoints,
      exitCode,
      buildOutput,
      lastValue,
      watches,
      exceptionFilters,
      start,
      stop,
      step,
      jumpTo,
      runTo,
      toggleBreakpoint,
      toggleExceptionFilter,
      setBreakpointCondition,
      selectFrame,
      evaluate,
      valueAt,
      query,
      addWatch,
      removeWatch,
    }),
    [
      status,
      message,
      offerInstall,
      program,
      capabilities,
      pausedLine,
      frames,
      variables,
      selectedFrame,
      breakpoints,
      exitCode,
      buildOutput,
      lastValue,
      watches,
      exceptionFilters,
      start,
      stop,
      step,
      jumpTo,
      runTo,
      toggleBreakpoint,
      toggleExceptionFilter,
      setBreakpointCondition,
      selectFrame,
      evaluate,
      valueAt,
      query,
      addWatch,
      removeWatch,
    ],
  );
}
