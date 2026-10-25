<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/PaneHeader.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The editor keeps its space](../../../../../contract.md)

Captured SHA-256: `1865746167127c14bd618bdd56aca2d19b01b5cfba57c3974345d4e165590c24`.

<book:entry path="apps/web/src/shell/PaneHeader.tsx">
import { createContext, useContext, useMemo, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import "./paneHeader.css";

type Part = "info" | "actions";
interface HeaderSlots {
  info: HTMLElement | null;
  actions: HTMLElement | null;
  setInfo: (node: HTMLDivElement | null) => void;
  setActions: (node: HTMLDivElement | null) => void;
}
const HeaderContext = createContext<HeaderSlots | null>(null);

/** Move only chrome into a pane's existing header. Its editor stays mounted. */
export function PaneHeaderProvider({ children }: { children: ReactNode }) {
  const [info, setInfo] = useState<HTMLDivElement | null>(null);
  const [actions, setActions] = useState<HTMLDivElement | null>(null);
  const slots = useMemo(() => ({ info, actions, setInfo, setActions }), [info, actions]);
  return <HeaderContext.Provider value={slots}>{children}</HeaderContext.Provider>;
}

export function PaneHeaderSlots() {
  const header = useContext(HeaderContext);
  return <div className="pane-header-slots" onPointerDown={event => event.stopPropagation()}>
    <div className="pane-header-info" ref={header?.setInfo} />
    <div className="pane-header-actions" ref={header?.setActions} />
  </div>;
}

/** Standalone/embedded surfaces retain their own chrome if there is no shell. */
export function PaneHeaderContent({ part, children }: { part: Part; children: ReactNode }) {
  const header = useContext(HeaderContext);
  return header?.[part] ? createPortal(children, header[part]!) : <>{children}</>;
}
</book:entry>

</book:doc>
