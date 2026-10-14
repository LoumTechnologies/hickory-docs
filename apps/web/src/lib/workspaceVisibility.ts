/** A retained workspace must not answer keys while Settings occupies the window. */
export function workspaceIsVisible() {
  return !document.querySelector("[data-workspace][hidden]");
}
export const SETTINGS_CLOSED = "hickory-settings-closed";
