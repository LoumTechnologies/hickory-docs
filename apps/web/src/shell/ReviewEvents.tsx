import { lazy, Suspense, useEffect, useState, useRef } from "react";
import { REVIEW_TARGET, REVIEW_ERROR } from "../lib/reviews";
const Review = lazy(() => import("../components/ReviewPanel").then(module => ({ default: module.ReviewPanel })));
export function ReviewEvents() {
  const [pending, setPending] = useState<{ type: string; detail: unknown } | null>(null);
  const [loaded, setLoaded] = useState(false);
  const replayed = useRef(false);
  useEffect(() => {
    if (loaded) return;
    const open = (event: Event) => { if (!replayed.current) setPending({ type: event.type, detail: (event as CustomEvent).detail }); };
    window.addEventListener(REVIEW_TARGET, open); window.addEventListener(REVIEW_ERROR, open);
    return () => { window.removeEventListener(REVIEW_TARGET, open); window.removeEventListener(REVIEW_ERROR, open); };
  }, [loaded]);
  return pending ? <Suspense fallback={<span role="status">Opening review…</span>}><Review initialOpen /><Replay pending={pending} onReady={() => { replayed.current = true; setLoaded(true); }} /></Suspense> : null;
}
function Replay({ pending, onReady }: { pending: { type: string; detail: unknown }; onReady: () => void }) {
  useEffect(() => { onReady(); window.dispatchEvent(new CustomEvent(pending.type, { detail: pending.detail })); }, [pending]);
  return null;
}
