// Minimal inline glyphs. No icon library and no image assets exist in this
// app — everything themes via `currentColor`, matching that convention.

/** The stop square — the universal "halt what is running" glyph. */
export function StopMark({ size = 14 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="currentColor"
      stroke="none"
      aria-hidden="true"
    >
      <rect x="5" y="5" width="14" height="14" rx="2" />
    </svg>
  );
}

export function TreeMark({ size = 18 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <path d="M12 21v-7" />
      <path d="M12 14l-4-4M12 14l4-4M12 10l-3-3M12 10l3-3M12 7V3" />
    </svg>
  );
}
