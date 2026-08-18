// Walking the attention queue — what ⌘J does, as one pure step.
//
// The queue's ORDER is the server's (hick_term::attention), so nothing here
// re-ranks anything. What is decided here is only where the next press lands,
// and the interesting case is the one a naive index gets wrong: between two
// presses the session you were looking at may have been answered, closed, or
// gone back to work, so an index into the old queue points at somebody else.
// Cursors are therefore ids, never positions.
//
// Protects docs/guarantees/terminal/the-attention-queue-ranks-by-claim.md

/**
 * The session ⌘J should land on next.
 *
 * `null` means nothing is waiting — which the caller says out loud rather
 * than swallowing, because "I pressed it and nothing happened" and "there is
 * nothing left to do" are the same keystroke and very different news.
 */
export function nextInQueue(queue: readonly string[], current: string | null): string | null {
  if (queue.length === 0) return null;
  const at = current === null ? -1 : queue.indexOf(current);
  // Not in the queue any more (answered, closed, or back to work): start
  // again at the front rather than guessing where it used to be.
  if (at < 0) return queue[0];
  return queue[(at + 1) % queue.length];
}

/** Whether the cursor still points at something that is waiting. */
export function stillWaiting(queue: readonly string[], current: string | null): boolean {
  return current !== null && queue.includes(current);
}
