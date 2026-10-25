<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/specs/freeform/tool-receipts-and-service-panes.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `f72ede609e1e0554f7c24b6180b80a7349522488e29a1276f4659597c50ade41`.

<book:entry path="docs/specs/freeform/tool-receipts-and-service-panes.md">
# Tool receipts and service panes

Audience: engineers designing Hickory’s API client and its connections to
agent conversations and document passages.

A tool can provide evidence independently of the model's account of its work.
A recorded file read establishes which snapshot and lines were returned to the
model. An answer's reference to those lines is a separate assertion that they
support the answer. Keeping both visible lets the reader inspect that assertion.

## Built: file reads

The agent conversation is an ordinary editor surface. Its recorded reads expose
context-family connections to file passages using the editor's ribbons or braces.
A completed read records the path, snapshot SHA-256, returned line range, and
repository commit when available. Native paths resolve from the document's
directory and are recorded relative to the workspace. ACP reads apply the same
range calculation to disk files, room files and unsaved editor buffers. Empty
responses have no invented source lines. Failed reads have no completed receipt.

A tool invocation records requested access; its location link says that it is a
request. A completed read is stronger evidence, and its explanation shows the
hash and commit. Adapter-reported locations without these receipts remain
adapter-reported evidence. Arbitrary shell output is not interpreted as a file
read. The current editor can differ from the recorded snapshot, so a ribbon to
its line range is navigation, not a verification that today's bytes match.

Editor context records what was supplied, including unsaved text, and accompanies
answer paragraphs even when the context record is folded. Untitled buffers use
explicit `hickory-buffer://<window>/<id>` editor identities rather than invented paths. Answer
citations attach to the paragraph that contains the reference, and stay declared
provenance: a model can omit, invent or misinterpret a reference. Recorded reads
prove inputs; they do not prove the model's interpretation or the truth of the
source material.

## Proposed: an API client with a request log

The service pane should be an Insomnia-style API client: a place to organize
saved requests, choose an environment, edit the method, URL, parameters, headers,
authentication and body, send a request, and inspect its response. Request and
response text use the editor experience, including native folding, selection,
search and provenance rails. The request log is part of this client, with a
workspace-wide view and filters for a specific API or service.

For example, an agent requests `GET /customers/42` and receives JSON containing
`"plan": "pro"`. Its tool call connects to that request's log entry. If the agent
answers “Customer 42 has the Pro plan,” its citation connects that answer
paragraph to the `plan` field in that particular recorded response. Clicking the
connection opens the service pane, selects the historical request and reveals
the field. The reader can inspect the request that produced it and the rest of
the response in the same pane.

Every send through the client creates a log entry, whether initiated manually,
by an agent tool, or by an instrumented execution path. The entry records the
initiator and its source location when available, service and operation,
request identity, observation time, method, resolved URL, captured request,
response status and headers, captured response, duration and content hashes.
Pending, completed, failed and cancelled requests remain distinguishable;
a failure keeps its error evidence even when there is no response body.
Retries, redirects and streamed responses need their actual attempts and
received data represented rather than overwritten by a final success.

Selecting an entry reveals the recorded request and response. Editing a saved
request changes the next request to send. Resending a historical request creates
a new entry; it preserves the original evidence. The log can be searched and
filtered by service, operation, initiator, time and outcome. “All requests” means
all requests through these capture paths. Calls made elsewhere require explicit
instrumentation; an unseen request must not be presented as recorded.

## Proposed: ribbons and braces into the log

An API request produces a receipt at the boundary that actually performs it.
It must not be inferred from a URL mentioned in an answer or from a command's
prose output. The tool invocation connects to the recorded request; the returned
response connects to the model input when it was actually supplied to the model.
An answer citation connects its paragraph to the supporting response passage.
A manually sent request becomes model context only if its contents are supplied
to the model later.

A target names a particular log entry, its request or response, and a text range
or structured location such as a JSON pointer that resolves within those bytes.
When open, the passage has the same ribbon or brace geometry as a document
passage. When closed, the connection lands on the service tab or request-log
entry; clicking opens the exact historical passage. Expanding a folded body
reveals its precise target using the normal editor's folding behavior.

A later response from the same API is a different value; it must never silently
replace the target of an older receipt. Status and a response hash establish an
observation, not the truth of everything the remote service asserted. An
outgoing request body can also carry provenance from the document or response
that supplied its bytes, when that derivation is actually recorded and checkable.

Request headers, authentication and response bodies can contain secrets. The
capture contract must define what is omitted or redacted before anything is
written to the session, and describe those omissions in the record. A hash alone
cannot reconstruct a response; a receipt whose body was not retained must not
offer a passage it cannot show. An ephemeral cache cannot be the sole basis of
a durable citation. The request log must retain the evidence that its links
name. If the user deletes an entry or its captured body, its links report missing
evidence rather than jumping to another request. Saved request definitions and
the historical log have separate lifetimes.

## Proposed: resolving non-file targets

When the service pane is built, the element-link seam should support a resource
identity, an observation/version identity and a location within it. The pane
resolver would provide the same geometry and navigation that file editors
provide today. API targets should not be encoded as pretend filesystem paths.
The concrete target representation can be settled with that first real pane;
there is no speculative service registry to implement now.

These targets use the existing provenance families. A response supplied to the
model is context; an answer citing it is declared; lineage requires an actual
checkable derivation. Adding a target type does not create another provenance
family. Capture and inspection remain on the user's machine and require no
Hickory-operated account, relay or telemetry.

The API client, request log, recorder and non-file target resolver are proposals. Only
the file and editor-context receipts described above are built.
</book:entry>

</book:doc>
