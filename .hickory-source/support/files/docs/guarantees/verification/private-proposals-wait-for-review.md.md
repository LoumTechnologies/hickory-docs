<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/verification/private-proposals-wait-for-review.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `df3932c952fe73a1a36714974af90699ad22e2b707f78491beb4bf60700dbcdb`.

<book:entry path="docs/guarantees/verification/private-proposals-wait-for-review.md">
# Private edit proposals open review before writing

An advertised private edit_doc call previews the captured source and waits for
independent host review before applying a source-bound edit. A prose proposal or
completion summary does not open that queue. The agent cannot supply its own
approval. The private catalogue advertises the supported whole="true" option,
latest whole-surface snapshot and complete replacement input, and identifies
run/after anchors as SHA-256 line hashes rather than source text. Invalid/stale
preview recovery exposes only fixed source/schema guidance, never raw host errors
or held-out target diagnostics. Existing exact-proposal freshness checks remain.

Private DeepSeek Python turns request the documented JSON-object response format
while retaining the 2048-token cap, strict action parser and tool authorization.
Ordinary DeepSeek requests and other provider transports keep their defaults.
JSON transport does not establish schema correctness or tool execution. A rejected
multi-object response executes nothing; repair feedback directs the model to the
first unfinished task step rather than skipping ahead.

The focused witness refreshes the non-editable installed worker from the current
locked project before launch. It reuses the actual configured interpreter. This
host currently uses Python 3.12.13, allowed by requires-python >=3.11; the preferred
3.14.7 download was unavailable to this uv installation. That is an explicit
runtime witness, not a claim that the preferred interpreter ran.

Origin: issue 72 real DeepSeek attempts diagnosed the change but did not submit
review, then supplied literal source as a hashline anchor. A later attempt opened
review without first executing required comparisons after a rejected response
containing multiple action objects. All those attempts failed acceptance and
remain retained; guidance/schema repairs do not turn them into passes.

Assumptions/alternatives: approval and predictor contexts remain separate; the
trusted host, not model text, owns persistence and execution. Loosening parsing,
automatically approving proposals or accepting a model's passing claim would
violate that boundary. Same-user filesystem compromise and opaque provider
behavior are outside this broker assurance. JSON-object output can still be
empty, truncated, invalid for the action schema or semantically wrong.
Falsifiers: a write before approval, stale proposal applied, rejected batch
executed, target values in refusal guidance, or a witness using stale installed
worker code while claiming to exercise a source fix.

LLM implementation review: partial, focused on catalogue/preview, installed-worker
policy and DeepSeek request construction. Not universal model reliability.
Executed checks: actual confined-worker accepted/declined review test passed
(2.19s); private whole-preview/no-write/no-exec/stale test passed; rejected-batch
Python test passed; JSON transport/cap/default test passed; cargo check passed;
scripted actual private engine/worker/browser sequence passed (30.1s/30.7s total).
Current JSON-mode real-provider witness is pending and is recorded separately in
the issue-72 handoff. Human judgment is absent; test-driver review is scripted.

Provider reference: [DeepSeek JSON output](https://api-docs.deepseek.com/guides/json_mode/).

Dependencies (SHA-256; changes stale this review):
- `apps/agent/src/hickory_agent_worker/instructions.py`: `7519daf89e8dd5a5a413ee61f0d2ba4d5a3e5a6d4da6f78513016a5d9af6190d`.
- `apps/agent/src/hickory_agent_worker/core.py`: `006c03fa696b38ef43da7e8f6386d49547ca9e87b1216f4aca6399093df36893`.
- `apps/agent/tests/test_worker.py`: `3b4d7f67dc09393ac3054285840b1f77611913df95e9526b107c4adeea49c1e3`.
- `crates/hickory-agent/src/llm_openai.rs`: `57c213a9ba67ca13e980a812bf9718113a57c45c78b96192883a58ab1deb3e4a`.
- `crates/hickory-agent/src/provider.rs`: `cbdb33fd51417d655b50e8a64cda812976c43b0c28b5ed9fb35706fde27ac49e`.
- `crates/hickory-cli/src/engine/mcp.rs`: `190bba110bbcae1f5df92774a796e893fc19bbec45f784f39734f6430b597e80`.
- `crates/hickory-cli/src/serve/private_predictions.rs`: `deed7f7acbd7fe99c5026fc32f303052a6eee7dfe51e4c26c4501c5f97091860`.
- `crates/hickory-cli/src/serve/private_prediction_tests.rs`: `ac1423a278a0ad2edfc8ac51461d59a1c3f973723ad1b6d86324ff62f97ad469`.
- `scripts/test-private-prediction-ui.sh`: `93efb6879a9181afb9149cc28dd700220340b62a961d6023dfa39bf4a7c3ecec`.

Native draft correction: an actual keyboard setup attempt exposed controlled
inputs reverting while asynchronous per-keystroke receipts were pending. The
rapid-typing challenge failed on the first character before the fix. Human draft
edits now update synchronously through editRequest, the same controller invoked
by registered agent commands. Agent intent/result receipts remain; Delegate and
Seal retain the authoritative persisted execution boundaries. Typing alone cannot
start a provider/runtime or approve a proposal. Ten focused component cases pass,
including rapid typing, separate agent field intents, recursive-delegation refusal
and ended-window closure. The first strict compile rejected an unsupported result
status; it was corrected to the established unavailable result.
- `apps/web/src/components/PrivatePredictionCoach.tsx`: `dc47ff382246a13a14e8e2b1ebaf25737da1fc88122e861a6dc98e6d9d85a54f`.
- `apps/web/src/components/PrivatePredictionCoach.test.tsx`: `938fd463f0c4e17203eca8c62dc88a1da78e73213b799acb6bc367e9708345e3`.
- `apps/web/e2e/privatePredictionCoach.spec.ts`: `635067a7c96b6a6c21b78d53a5b3b777a90b0bbf0d5ce242265e3794f0c8b502`.

Real-provider follow-up: the 47353e3c DeepSeek engine/worker/DAP/browser flow
passed (1.1m/1.2m total) after the documented failures. It retained the original
failures, corrected source, unchanged expectations, promotion revalidation and
changed-input refusal. The scenario-bound fixture still enforces the original
requirements and retains unused/repeated seals; scripted mode keeps exactly six
attempts, while live mode bounds and preserves extra attempts. Automated review
is not human judgment. Exact draft/evidence hashes are in the October 23 handoff.


ACP harness follow-up (partial LLM review, not universal certification): explicit
private-only live profiles are supported; missing configuration refuses before
setup. Scripted ACP passes 18.4s/19.1s, shell syntax and strict changed-spec
TypeScript pass. The real ACP run does not pass; the October 23 handoff retains
its completed DeepSeek responses and blocked unconfigured fallback. No human
judgment or native/provider footage is established. Earlier dependency hashes
remain historical; these changed dependencies supersede their earlier hashes:
- `scripts/test-private-prediction-ui.sh`: `da2324f4d8d782e27746bc648a163bcf0444beef0606288c7d3e0fb110b9714d`.
- `apps/web/e2e/privatePredictionCoach.spec.ts`: `cdeb407e2c2f257b4337111d7c1cc517e395dd587da881bd83d2fd6aebcff65a`.
</book:entry>

</book:doc>
