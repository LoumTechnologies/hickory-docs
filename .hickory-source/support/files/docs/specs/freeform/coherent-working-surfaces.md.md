<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/specs/freeform/coherent-working-surfaces.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `995461a987b939a54e48165ff9c8b0e54bd6dc9210841826f19e4482ae8e8347`.

<book:entry path="docs/specs/freeform/coherent-working-surfaces.md">
# Coherent working surfaces

Adopted 2026-10-08 in response to the request to implement the seven findings
from the product coherence review. These rules describe user-visible behavior;
they do not change document syntax, execution backends, or provenance families.

1. **Saved means acknowledged file bytes.** Documents and ordinary files save
   automatically. A literate view saves explicitly. Each becomes clean only
   when its current bytes are acknowledged at its write destination. Explicit
   Save flushes pending work; it is not a separate checkpoint. Recovery drafts
   are outside the project and never count as file saves. Untitled remains
   unnamed until Save. A later edit cannot be cleared by an earlier response.
2. **Conflict choices name their effects.** Overwrite disk with this buffer,
   reload from disk, leave disk unchanged, and regenerate from document are
   distinct actions. A dismissed warning is not a resolved divergence.
3. **Action failures preserve the working surface.** Run, Test, rendering, and
   saving failures appear beside the affected document or file with a retry.
   Only a failure to load a document can replace its editor. Installation
   remains capability-dependent; a missing platform SDK is a download link.
4. **A view declares its contract.** Its identity, editability, and write
   destination are visible together. Historical readings write nothing.
   Generated-file editable regions write into their owner. Literate views
   write into their backing files or document and keep an arrangement locally.
   History actions still obey the publication floor.
5. **Placement follows scope.** Cell actions stay on the rail; tab actions and
   results stay in the tab; workspace notices stay at workspace scope. Primary
   actions remain visible and occasional actions live in More actions.
   Feature settings stay next to the feature, with consistent names in Settings.
6. **Results and evidence are distinct.** Recorded success, recorded failure,
   stale recording, and unrecorded state say what happened. Matching expected
   stdout and passing diagram assertions say what was checked. A stale AI
   explanation is not an execution failure or a statement of semantic truth.
7. **Inspect, write, report.** Insert, scaffolding, environment commands,
   filesystem Dry Run, agent proposals, and literate-view saves expose proposed
   bytes or effects. Previews do not write. Results and recovery remain beside
   the work. Plain keystrokes still autosave; this rule introduces no per-edit
   confirmation. A preview is not evidence that execution will succeed.

## Scope and consequential assumptions

The existing engine remains the authority for persistence and concurrency.
GET document reads file bytes; rendering is independent of that acknowledgment.
Revision/hash checks remain the authority for source conflicts. No new server,
account, telemetry, or remote execution is introduced.

Literate and generated views now participate in close prompts, dirty-tab marks,
and Save All, including while inactive. Their pending text survives tab switches
in the current window; they do not gain crash-recovery drafts. Close prompts
explicitly identify that limit rather than borrowing the document preference.
The existing recovery preference remains for named documents and ordinary files;
Untitled recovery remains independent.

The former explicit-Save checkpoint for document dirty state is superseded.
Its motivating recovery distinction still holds: a recovery draft and an
acknowledged write to the actual document are different events. The new contract
chooses the latter as the meaning of Saved consistently across working surfaces.

## Validation

Behavioral contracts live in `docs/guarantees/`. Unit tests deliberately delay
save replies, type newer edits, refuse writes, and close inactive views. Browser
acceptance runs against an isolated real engine and git repository. It also
injects a Run request failure to verify that retry preserves editable text.
Automated checks do not establish human usability or cover every native OS.
</book:entry>

</book:doc>
