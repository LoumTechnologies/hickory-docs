# Capture the agent-first native workflow

Audience: engineers recording a supported native build with a real configured
provider for issue 72. Retain the exact build revision and platform. Install
Node.js and the local execution dependencies before starting. Release asset
existence does not establish the workflow works.

1. Open **AI engineer tour** and explicitly create an isolated practice repository.
   Show its new window/path and the transparent seeded meeting/proposal labels.
   Demonstrate the anchored coach’s Locate, dismissal and reopening with normal
   keyboard focus.
2. Copy the investigation task into the normal workspace conversation. Show the
   provider and explicitly Send. Let the agent inspect/setup the environment,
   reproduce the failure, bind a breakpoint and inspect a real paused stop through
   disclosed UI actions. Show source/run/stop/request identities. Fix actual
   errors; never substitute an example response.
3. Interpret two 80ms attempts consuming 160ms on the seeded source. The human
   selects the missed qualification and creates a real review thread requiring
   one total deadline including retries. Show the criterion in normal controls.
4. Copy and Send the correction task. Independently review and approve or replace
   the actual preview. Let the agent weave as necessary, invoke Verify saved
   document and inspect its receipt. Match the current root hash and retain the
   short-attempt and empty-sequence checks. Budget allocation does not establish
   real network or OS timing.
5. Review the diff/evidence and resolve the real thread yourself. Close/reopen the
   coach to demonstrate reconciliation. Skipped or missing witnesses must remain
   unestablished; saved progress cannot supply evidence.
6. Load reviewed publication JSON and compare source identity and tested revision.
   Preserve stale/failed results. Publication, local execution and deployment are
   separate facts.

Optionally profile hotspot.md through Runtime observations before and after a
reviewed optimization. Show separate source/run IDs, call counts, timing scope and
profiler overhead. Do not promise a fixed speedup.

## Publish reviewed footage

Review footage and any redacted public session export for private content. Keep
raw sessions/ files private. Add the reviewed video and optional export to the
existing publications/selected.json entry’s assets, with their SHA-256 values in
reviewed_inputs. Add demonstration metadata:

```json
{
  "reviewed": true,
  "capture_kind": "native-provider",
  "captured_revision": "<40 lowercase hexadecimal characters>",
  "recording_asset": "<selected .mp4 or .webm path>",
  "cuts": ["Describe every omitted wait or edit"]
}
```

An optional session_asset must name a separately selected public export. Reuse the
existing publication command and CI path. The homepage verifies selected media
bytes before creating a playback URL; absent/invalid footage stays unavailable.

Retain the deployed URL, Build source revision, publication tested revision and
native build revision. Inspect playback, downloads, browser example and install
links on the deployed page. Record mismatches as failures.

This implementation session has not supplied a native/provider recording or a
matching deployed-build witness. Broad regression and post-commit monitoring
belong to the operator: use just checks-status and
[GitHub Actions](https://github.com/LoumTechnologies/hickory-docs/actions).
