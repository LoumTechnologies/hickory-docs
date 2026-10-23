# Record the issue 72 native workflow

For Loum, recording the existing Linux build with real DeepSeek and real human review.
No additional key or credit setup is expected on this machine. The helper reads
only DEEPSEEK_API_KEY from the environment or repository .env, without printing it.

## Start a fresh take

From the repository root:

```sh
python3 scripts/prepare-issue72-recording.py --launch
```

The helper verifies the exact witnessed .deb checksum, unpacks it without
installation, and prints a new recording directory and request.txt path. It
configures the existing Python worker/debugpy runtime and isolated app state.
It launches the app but sends no model prompt. Keep that directory: it contains
manifest.json, native.log and the retained state. Every invocation creates a new
take; failed histories are preserved. The native functional revision is
47353e3c, package version 0.1.0-issue72.47353e3c, Linux amd64. Later ACP-only
changes are not in this Python recording build; record this exact identity.

The helper was prepared successfully without launching or provider calls at
/tmp/hickory-issue72-human-recording-ready. Package tampering and reuse of an
existing take directory were refused in focused safety checks. Starting the
command above creates your own fresh take.

## Record one continuous investigation

1. Start your window recorder. In Hickory open **AI engineer tour**, then
   **Create a private prediction practice**. Record the new private window.
   **Dismiss coach** can uncover fields; **Resume coach** restores guidance.
2. Set **Provider** to deepseek and **Model** to deepseek-chat. Leave
   **ACP adapter (optional)** blank. Paste the printed request.txt into
   **Investigation request**. It contains required expectations, not observed
   answers. Press **Delegate investigation** once.
3. Let the agent read source, seal the first prediction before launch, seal the
   next prediction before Continue, and retain the non-passing combined check.
   Do not run those steps for it or tell it the actual target values. A model
   explanation alone is not a completed comparison.
4. When **Document change review** opens, read the diff yourself. Accept only
   the remaining-budget correction (allowance = budget - elapsed, optionally
   clamped at zero) and harmless blank-line cleanup. Preserve capture location,
   inputs, requirement and original expectations. Reject an unrelated rewrite;
   retain the failed take if the agent cannot recover. Do not silently repair
   its proposal by hand for this demonstration.
5. Let the agent repeat the unchanged initial/future debugger and combined
   predictions against the repaired source. Show retained history containing
   both the original failures and corrected passes. Repeated attempts may
   remain; do not remove them to manufacture a six-attempt story.
6. Once investigation finishes, press **Revalidate and promote**. Record the
   host revalidation/source identity and passing original requirement. A
   disabled button, stale source or failed revalidation is a failed take.
7. Press **End private practice**, then **Close window**. Stop the recording.
   Keep the original file uncut. Note any later cuts explicitly.

You supply review judgment and recording; the agent supplies the investigation,
sealing, execution and proposed correction. Never paste human-only actuals into
its prompt. Private logs/state are diagnostic material, not public exports.

## Return a review package

Keep the video beside manifest.json. Record its SHA-256, what you accepted,
whether the sequence above passed, and every omission/cut. Share the local take
directory with the next continuation. Do not mark demonstration.reviewed true
until you have inspected the actual video for private content. The agent will
prepare selected public assets through the existing publication pipeline after
your review; no automatic upload occurs.

If the window fields still cannot be edited manually, stop and retain that take:
that would establish an application bug beyond the failed automation. If the
provider fails, keep the failure and report the visible error; repeated paid
retries are not required. Stop a running investigation before End. Do not delete
its quarantine/state or use ordinary Run to bypass the private workflow.

Broad regressions and post-commit checks remain yours: just checks-status and
[GitHub Actions](https://github.com/LoumTechnologies/hickory-docs/actions).
