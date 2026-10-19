# Record and transcribe a meeting locally

Open a project, expand **Import meeting transcript → Record a meeting**, and
inspect the application-data save location. Microphone capture asks for browser
permission. System audio is unavailable. Stop capture before transcription.
Audio stays outside the project until explicitly deleted, with a 256 MiB limit
per recording. **Save retained audio** exports it; **Recover and stop retained
audio** finalizes an interrupted capture using its actual retained bytes.

The first local adapter is `scripts/transcribe-meeting-local.py`, using
[Faster Whisper](https://github.com/SYSTRAN/faster-whisper). Install Faster Whisper
in a separate Python environment and provision a complete CTranslate2 model
locally, including `tokenizer.json`. These are explicit installation steps; the
application does not download a model. Configure:

- Processing: Local adapter.
- Executable: the absolute path to that environment's Python.
- Arguments: `["/absolute/path/to/scripts/transcribe-meeting-local.py", "--model", "{model}", "{audio}"]`.
- Model/settings identity: the absolute local model directory.

The supplied adapter uses CPU int8 inference and beam size 5, emits VTT with model
estimated timestamps, and invents no speaker identity. It requires a local
model/tokenizer and sets offline library flags. A different local executable is
a user-selected program with its own permissions and network behavior.

An optional remote adapter accepts a raw audio POST with its content type and
`X-Hickory-Model` setting, returning UTF-8 VTT/SRT/SBV or speaker-labelled text.
This is a documented adapter protocol, not a direct commercial-provider API.
Configure an HTTPS adapter you control. Each transcription needs fresh consent;
cancellation cannot undo a remote upload or provider charges already incurred.
Redirects are refused. No credentials are added automatically.

Review the transcription and corrections before importing. The existing importer
retains corrected text; local sidecars separately retain original transcription,
audio SHA-256, transcript SHA-256, correction SHA-256 and adapter settings. Imported
notes reference that evidence without adding audio to Git. Deleting local evidence
leaves the imported note and its historical hashes in place.

Validation here covers storage, refusal and consent/failure UI behavior. A real
microphone and a provisioned model must be exercised on the target platform
before claiming #60 acceptance. The implementation environment had neither an
audio device nor an installed speech model; no remote audio was sent.
