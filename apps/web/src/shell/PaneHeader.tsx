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
