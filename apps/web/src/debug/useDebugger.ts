// One debug session per open document, and the editor state that follows it.
//
// The hook owns three things the UI reads and the editor draws: where
// execution is paused, what is in scope there, and which breakpoints the
// adapter could actually bind. Everything arrives as an event, because a
// `continue` takes as long as the program does.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Realtime } from "../api/realtime";
import { DebugClient } from "./client";
import type {
  BreakpointStatus,
  DebugCapabilities,
  DebugEvent,
  Frame,
  Step,
  Variable,
} from "./client";

export type DebugStatus = "idle" | "starting" | "paused" | "running" | "finished" | "failed";

export interface DebugSession {
  status: DebugStatus;
  message: string | null;
  capabilities: DebugCapabilities | null;
  /** 0-based document line execution is paused on. */
  pausedLine: number | null;
  frames: Frame[];
  variables: Variable[];
  selectedFrame: number | null;
  /** Where the user has asked to stop, whether or not it bound. */
  breakpoints: BreakpointStatus[];
  lastValue: { expression: string; value: string; type: string | null } | null;

  start(): void;
  stop(): void;
  step(how: Step): void;
  jumpTo(line: number): void;
  runTo(line: number): void;
  toggleBreakpoint(line: number): void;
  selectFrame(id: number): void;
  evaluate(expression: string, context?: "hover" | "watch" | "repl"): void;
  /** Ask for a value and get it back, for the hover tooltip. */
  valueAt(expression: string): Promise<string | null>;
}

export function useDebugger(realtime: Realtime, docPath: string): DebugSession {
  const [status, setStatus] = useState<DebugStatus>("idle");
  const [message, setMessage] = useState<string | null>(null);
  const [capabilities, setCapabilities] = useState<DebugCapabilities | null>(null);
  const [pausedLine, setPausedLine] = useState<number | null>(null);
  const [frames, setFrames] = useState<Frame[]>([]);
  const [variables, setVariables] = useState<Variable[]>([]);
  const [selectedFrame, setSelectedFrame] = useState<number | null>(null);
  const [breakpoints, setBreakpoints] = useState<BreakpointStatus[]>([]);
  const [lastValue, setLastValue] =
    useState<{ expression: string; value: string; type: string | null } | null>(null);

  const sessionRef = useRef<string | null>(null);
  // Hover asks for a value and wants it back; the channel answers with an
  // event, so the request is parked here until its answer arrives.
  const pendingValues = useRef(new Map<string, (value: string | null) => void>());

  const uri = useMemo(() => `hick:///${docPath.replace(/^\/+/, "")}`, [docPath]);
  const client = useMemo(() => {
    const channel = realtime.debug?.();
    return channel ? new DebugClient(channel) : null;
  }, [realtime]);

  useEffect(() => {
    if (!client) return;
    const off = client.on((event: DebugEvent) => {
      switch (event.event) {
        case "started":
          sessionRef.current = event.session;
          setCapabilities(event.capabilities);
          setBreakpoints(event.breakpoints);
          setStatus("running");
          setMessage(null);
          break;
        case "stopped":
          setStatus("paused");
          setPausedLine(event.line);
          setFrames(event.frames);
          setVariables(event.variables);
          setSelectedFrame(event.frames[0]?.id ?? null);
          setMessage(null);
          break;
        case "breakpoints":
          setBreakpoints(event.breakpoints);
          break;
        case "value": {
          setLastValue({ expression: event.expression, value: event.value, type: event.type });
          const waiting = pendingValues.current.get(event.expression);
          if (waiting) {
            waiting(event.value);
            pendingValues.current.delete(event.expression);
          }
          break;
        }
        case "finished":
          setStatus("finished");
          setPausedLine(null);
          setFrames([]);
          setVariables([]);
          break;
        case "ended":
          sessionRef.current = null;
          setStatus("idle");
          setPausedLine(null);
          setFrames([]);
          setVariables([]);
          setCapabilities(null);
          break;
        case "failed": {
          setMessage(event.message);
          // A failed STEP leaves the program where it was — still paused —
          // so only a failure with no session at all is fatal to the UI.
          if (!sessionRef.current) setStatus("failed");
          const stillWaiting = [...pendingValues.current.values()];
          pendingValues.current.clear();
          for (const resolve of stillWaiting) resolve(null);
          break;
        }
      }
    });
    return off;
  }, [client]);

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
            ? // Optimistically verified: the adapter's answer replaces this a
              // moment later, and a dot that appears only after a round trip
              // feels broken.
              [...current, { line, verified: true }]
            : without;
        const session = sessionRef.current;
        if (client && session) {
          client.setBreakpoints(
            session,
            next.map((breakpoint) => ({ line: breakpoint.line })),
          );
        }
        return next.sort((a, b) => a.line - b.line);
      });
    },
    [client],
  );

  const start = useCallback(() => {
    if (!client) return;
    setStatus("starting");
    setMessage(null);
    client.start(
      uri,
      breakpoints.map((breakpoint) => ({ line: breakpoint.line })),
    );
  }, [client, uri, breakpoints]);

  const stop = useCallback(() => {
    if (client && sessionRef.current) client.stop(sessionRef.current);
  }, [client]);

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
    [client, status, selectedFrame],
  );

  const selectFrame = useCallback(
    (id: number) => {
      setSelectedFrame(id);
      const frame = frames.find((candidate) => candidate.id === id);
      // Selecting a frame moves the paused marker with it, because the line
      // you are looking at should be the line whose values you are reading.
      if (frame?.line !== undefined) setPausedLine(frame.line);
    },
    [frames],
  );

  return {
    status,
    message,
    capabilities,
    pausedLine,
    frames,
    variables,
    selectedFrame,
    breakpoints,
    lastValue,
    start,
    stop,
    step,
    jumpTo,
    runTo,
    toggleBreakpoint,
    selectFrame,
    evaluate,
    valueAt,
  };
}
