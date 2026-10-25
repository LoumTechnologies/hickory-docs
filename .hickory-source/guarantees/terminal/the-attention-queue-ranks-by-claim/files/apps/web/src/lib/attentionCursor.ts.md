<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/attentionCursor.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The Attention Queue Ranks By Claim](../../../../../contract.md)

Captured SHA-256: `cef22ee8aa54de47d1f59af829a39acd183e5e623234c41c17328813b7a92730`.

<book:entry path="apps/web/src/lib/attentionCursor.ts">
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
</book:entry>

</book:doc>
