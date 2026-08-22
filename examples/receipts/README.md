# Receipts: a Slack message whose every sentence has a source

Two folders, one scenario, built two ways. A fake meeting (`inbox/*.vtt`)
is ingested into a note; an analysis document computes the numbers from
Priya's export in a pinned cell; a fix document tangles and tests the code
Sam proposed; a message document drafts what Nate will post in `#eng`, one
sentence per fragment, with a model-written check under each sentence and a
`slack:claim` on the one that is a judgment. The Slack text is a generated
file, so lineage — and the app's ribbons — trace every sentence back through
the findings to the meeting turn.

| Folder | Authored by | Sessions |
|---|---|---|
| `claude-code/` | Claude Code, writing the documents and running the CLI | — |
| `hick-agent/` | `hick agent`, one prompt per document, from stubs | `hick-agent/sessions/` (five; the first two fail instructively) |

Both verify: `hick test examples/receipts`.

## Walk it

```sh
hick lineage examples/receipts/claude-code/message.hick --output messages/2026-08-24-eng-reply.txt
#  48..181  paste  …/meetings/2026-08-20-checkout-latency-sync.hick bytes 1408..1541   ← Sam's turn
hick test examples/receipts/claude-code            # checks + expectations + drift, no tokens
```

In the app: open `message.hick`, open `messages/2026-08-24-eng-reply.txt`
from the port in the divider, hover the quoted line — the ribbon reaches the
meeting note's tab (`ribbons-across-documents.png`); click it to go there.

Change a fact — make Sam say "forty-five" in the `.vtt`-derived transcript
block, or edit a finding in `analysis.hick` — and `hick test` reports the
check under the affected sentence STALE until `hick refresh` re-checks it.

## Rebuild from nothing

```sh
cd examples/receipts/claude-code
hick ingest .                                   # inbox/*.vtt → a note; moves the source to inbox/ingested/
hick refresh meetings                           # summary + action items (needs a provider key)
hick run . && hick refresh message.hick         # cells, files, then the four checks
hick test .
```

The agent variant was driven with `hick agent --dir . --doc <stub> "<prompt>"`;
the prompts are the first `<hick:user>` of each session file. The message
it drafted invented its numbers, and the checks said so — read
`hick-agent/message.hick` and then
`docs/specs/freeform/receipts-for-a-message.md` for what this test fixed and
what it found still painful.
