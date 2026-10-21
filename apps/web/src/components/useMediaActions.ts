import { useEffect, useId, useRef, useState, type RefObject } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome, UiValue } from "../lib/uiActions";

/** Standard media controls operate on the actual loaded element, never DOM clicks. */
export function useMediaActions(ref: RefObject<HTMLVideoElement>, identity: string) {
  const parent = `media:${useId()}`, [, render] = useState(0);
  const lifetime = useRef<{ element: HTMLVideoElement | null; id: string }>({ element: null, id: "" });
  const node = () => {
    if (lifetime.current.element !== ref.current) lifetime.current = { element: ref.current, id: crypto.randomUUID() };
    return lifetime.current;
  };
  const revision = useUiRevision(() => { const owner = node(); return [identity, owner.id, owner.element?.currentSrc, owner.element?.duration, owner.element?.error?.code]; });
  useEffect(() => {
    const media = ref.current; if (!media) return;
    const update = () => render(value => value + 1);
    const events = ["loadedmetadata", "durationchange", "play", "pause", "ended", "timeupdate", "volumechange", "ratechange", "error", "progress"];
    for (const event of events) media.addEventListener(event, update);
    return () => { for (const event of events) media.removeEventListener(event, update); };
  }, [ref, identity]);
  const media = ref.current;
  useUiReading("recorded media status", media ? { identity, paused: media.paused, ended: media.ended, time: media.currentTime,
    duration: Number.isFinite(media.duration) ? media.duration : null, volume: media.volume, muted: media.muted, playback_rate: media.playbackRate, loop: media.loop,
    ready_state: media.readyState, network_state: media.networkState, error: media.error ? { code: media.error.code, message: media.error.message } : null,
    captions: Array.from(media.textTracks ?? []).map((track, index) => ({ index, kind: track.kind, label: track.label, language: track.language, mode: track.mode })),
    fullscreen: document.fullscreenElement === media, picture_in_picture: document.pictureInPictureElement === media,
    scope: "Actual browser media element state; historical footage is not live execution or proof that a person watched it." } : { identity, loaded: false });
  const command = (key: string, label: string, parameters: UiCommand["parameters"], run: (element: HTMLVideoElement, args: Record<string, UiValue>) => Promise<void> | void) => ({ key: `${parent}:${key}`, command: {
    label, parent, subject: parent, parameters, prerequisites: ["Browser permissions/user activation may be required; failures report an addressable input request."],
    read: () => ({ revision: revision(), enabled: !!ref.current, reason: ref.current ? undefined : "Load the selected reviewed recording first" }),
    execute: async (args: Record<string, UiValue>, _actor: unknown, signal: AbortSignal): Promise<UiOutcome> => {
      const element = ref.current;
      if (!element || signal.aborted) return { status: "unavailable", reason: "The originating media control is unavailable or cancelled" };
      try {
        await run(element, args);
        if (ref.current !== element || signal.aborted) return { status: "unavailable", reason: "Media changed or cancellation arrived after the browser operation; inspect current playback, no rollback is asserted" };
        return { status: "completed", value: { media_callback_completed: true, identity, watched_or_heard: "not established" } };
      } catch (error) { return { status: "requires_input", reason: `Browser media operation refused: ${String(error)}. Use the visible media control to supply browser activation/permission, then inspect current playback and retry.`, request: `${parent}:${key}` }; }
    },
  } });
  useUiCommands([
    command("play", "Play reviewed recording", {}, element => element.play()),
    command("pause", "Pause reviewed recording", {}, element => element.pause()),
    command("seek", "Seek reviewed recording", { seconds: { type: "number", minimum: 0, maximum: 1e9 } }, (element, args) => {
      if (!Number.isFinite(element.duration) || (args.seconds as number) > element.duration) throw Error("Seek requires loaded metadata and a time inside this recording");
      element.currentTime = args.seconds as number;
    }),
    command("sound", "Set reviewed recording sound", { volume: { type: "number", minimum: 0, maximum: 1 }, muted: { type: "boolean" } }, (element, args) => { element.volume = args.volume as number; element.muted = args.muted as boolean; }),
    command("rate", "Set reviewed recording playback rate", { rate: { type: "number", minimum: 0.25, maximum: 4 } }, (element, args) => { element.playbackRate = args.rate as number; }),
    command("loop", "Set reviewed recording looping", { loop: { type: "boolean" } }, (element, args) => { element.loop = args.loop as boolean; }),
    command("captions", "Set reviewed recording caption track", { track: { type: "number", minimum: 0, maximum: 1000, integer: true }, mode: { type: "string", maxLength: 8, choices: ["disabled", "hidden", "showing"] } }, (element, args) => {
      const track = element.textTracks[args.track as number]; if (!track) throw Error("This recording has no such caption track"); track.mode = args.mode as TextTrackMode;
    }),
    command("fullscreen", "Set reviewed recording fullscreen", { shown: { type: "boolean" } }, async (element, args) => {
      if (args.shown) await element.requestFullscreen(); else if (document.fullscreenElement === element) await document.exitFullscreen();
    }),
    command("pip", "Set reviewed recording picture in picture", { shown: { type: "boolean" } }, async (element, args) => {
      if (args.shown) await element.requestPictureInPicture(); else if (document.pictureInPictureElement === element) await document.exitPictureInPicture();
    }),
  ]);
}
