export const RUNTIME_REVEAL = "hickory:runtime-reveal";
export function revealRuntimeEvent(run: string, event: string) {
  window.dispatchEvent(new CustomEvent(RUNTIME_REVEAL, { detail: { run, event } }));
}
