<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/UiLink.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2e99708b2750"></a>
## `apps/web/src/components/UiLink.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-2e99708b2750-requestLinkNavigation"></a>
### `requestLinkNavigation`

<storycode:copy id="f-2e99708b2750-requestLinkNavigation">
import { useId, type AnchorHTMLAttributes, type MouseEvent } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiOutcome } from "../lib/uiActions";

/** The navigation boundary shared by anchors and semantic agent requests. */
export function requestLinkNavigation(href: string, target = "_self"): UiOutcome {
  const url = new URL(href, location.href);
  if (!["http:", "https:", "blob:"].includes(url.protocol) || url.protocol === "blob:" && url.origin !== location.origin) return { status: "unavailable", reason: "This link does not have an approved browser navigation scheme/origin" };
  const attachment = new URL(location.href).searchParams.get("parentOrigin");
  if (attachment && window.parent !== window && url.origin === location.origin) url.searchParams.set("parentOrigin", attachment);
  if (target === "_blank") {
    const opened = window.open(url.href, "_blank", "noopener");
    if (!opened) return { status: "requires_input", reason: "The browser returned no window handle. The link may have opened with opener isolation, or a popup may have been blocked. Inspect the destination or activate the visible link before retrying.", request: "browser-link-activation" };

  } else window.location.assign(url.href);
  return { status: "completed", value: { navigation_requested: true, destination: url.href, loaded: "not observed" } };
}

</storycode:copy>
<a id="f-2e99708b2750-UiLink"></a>
### `UiLink`

<storycode:copy id="f-2e99708b2750-UiLink">
export function UiLink({ href = "", children, onClick, download, target, ...props }: AnchorHTMLAttributes<HTMLAnchorElement>) {
  const id = `link:${useId()}`;
  const label = props["aria-label"] ?? (typeof children === "string" ? children : props.title ?? "Open linked evidence");
  const revision = useUiRevision(() => [href, target, download, label]);
  const activate = (): UiOutcome => download !== undefined && download !== false
    ? { status: "requires_input", reason: `Download ${String(download || label)} using the visible browser link. Standard anchor downloads require the browser's activation/download UI; file persistence has not been observed.`, request: id }
    : requestLinkNavigation(href, target);
  useUiCommands([{ key: id, command: { label, parent: "browser-links", subject: id, parameters: {}, prerequisites: download !== undefined ? ["Browser download activation is required; no automatic upload or persistence claim."] : [],
    read: () => ({ revision: revision(), enabled: !!href, value: { href, target: target ?? "_self", download: download === undefined ? null : String(download) } }), execute: async () => activate(),
  } }]);
  const click = (event: MouseEvent<HTMLAnchorElement>) => {
    onClick?.(event);
    if (event.defaultPrevented || event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey || download !== undefined && download !== false) return;
    event.preventDefault(); activate();
  };
  return <a {...props} href={href} target={target} download={download} onClick={click}>{children}</a>;
}
</storycode:copy>

### Assemble `apps/web/src/components/UiLink.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/UiLink.tsx"><storycode:paste select="#f-2e99708b2750-requestLinkNavigation"/><storycode:paste select="#f-2e99708b2750-UiLink"/></storycode:file>

</storycode:doc>
