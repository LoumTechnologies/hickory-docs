# Deliver a verified final changeset from an ACP agent

For adapter authors integrating an ACP v1 agent with Hickory's declared literate
projects. This contract lets the agent repair its final deliverable before its
turn is reported complete. It adds no write permission or execution sandbox.

## Complete one turn

1. On `initialize`, recognize the client's capability and advertise the same
   version in `agentCapabilities._meta`:

   ```json
   {"hickorydocs.com/changesets":{"version":1}}
   ```

2. When `session/prompt._meta["hickorydocs.com/changesets"].required` is true,
   retain its `boundary`. Hickory has already captured A, including dirty output
   bytes. Work normally through the configured tools.
3. Call MCP `changeset_status` on the supplied `hick` server. The result contains
   `boundary`, `startingFingerprint`, current `expected` fingerprint, output
   `differences`, and a replayable `response.md` template. Pass
   `include_snapshots: true` to also receive complete `starting` and `current`
   byte snapshots; the default avoids sending unchanged code twice. A template refusal does not prevent a
   supported explicit construction from being submitted.
4. Replace placeholder prose with the concise successful explanation. Submit:

   ```json
   {
     "name":"submit_changeset",
     "arguments":{
       "boundary":"TURN_FROM_PROMPT",
       "expected":"FINGERPRINT_FROM_STATUS",
       "entry":"response.md",
       "sources":{"response.md":"COMPLETE LITERATE DOCUMENT"}
     }
   }
   ```

5. Inspect the JSON result inside the MCP text content. `accepted: false`
   includes differences and repair guidance; tool errors carry validation or
   stale-state diagnostics. Repair and resubmit. Read status again after further
   output writes, including a build that changes captured outputs.
6. End the turn only after acceptance. Hickory rechecks B at completion. It sends
   at most three repair prompts if an `end_turn` arrives without acceptance.
   A non-completion stop reason is not successful artifact completion.

The document executes against captured A in the portable project evaluator.
Verification does not launch commands. Supply every included `.md` in `sources`;
verification cannot fetch missing documents from the working tree. Full entry
comparison detects omitted files and changed modes. Acceptance validates outputs,
not whether the explanation accurately describes why the change was made.

## Use ACP directly

After negotiating version 1, the agent can issue the client request
`_hickorydocs.com/changeset`. Every request requires the current `sessionId` and
`boundary`. Set `operation` to `status` or `submit`. Status accepts the same
`include_snapshots` flag as MCP. Submit uses the same
`expected`, `entry`, and `sources` fields as MCP. Both paths share acceptance
state and recording. Unknown sessions and expired turn boundaries are refused.
The capability is also exposed in Hickory's ACP connection snapshot.

Capability absence or an unsupported version leaves ordinary ACP behavior in
place. Do not assume the standard Codex or Claude adapters implement this
extension. They can use the MCP tools voluntarily; mandatory finalization needs
adapter participation. ACP v2 is not implemented here.

## Use standalone MCP

Call `begin_changeset` with `doc` before editing a declared project. The MCP
server captures A and refuses a second begin so a failed artifact cannot be
"fixed" by recapturing B as A. Then use status and submission above. This
explicit task boundary survives engine forwarding for that MCP client, but not
an engine/server restart; standalone MCP cannot enforce when another harness
ends its turn.

## Review and debug the result

Hickory records accepted artifacts separately from trial work and renders their
patches with captured whole-project context. Source equality does not resolve
ambiguous native-write attribution. Resolve that separately before incorporating
into maintained source. Accepted artifacts are immutable; editable final-artifact
fork/apply workflows remain future work.

Run `just test-agent-changesets` for the real ACP/MCP peer and verifier cases,
and `just test-agent-changesets-web` for final-document rendering and proposal UI.
The peer is deterministic and tests the wire contract; it cannot establish
real-provider explanation quality or third-party adapter compliance.
