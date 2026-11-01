# Qwen3.8-27B on the local CPU — 1 November 2026

For maintainers deciding whether the newly downloaded model is usable on the 32 GiB Ryzen 5 3600 machine. This is a small feasibility comparison, not a general model ranking.

Removed about 50.3 GB of rebuildable Rust caches: the ignored `hickory-docs-literate/target/debug` directory (release build retained), and `pilockdb-literate/target/debug/incremental` (binaries and dependencies retained). Tracked files were preserved. Downloaded `qwen3.8:27b` through Ollama. The model manifest and identity are retained in the evidence.

The document loop uses the same three tasks, instructions, evaluator version and budgets as the earlier Qwen3-Coder pilot: CPU execution, six threads, 8192-token context, 1536-token output cap, temperature 0.2, top_p 0.95, top_k 20, seed 42 and thinking disabled. A short readiness probe loads the model before fixture timing.

| Task | Model | Completed | Held-out checks | Iterations | Output tokens | Decode t/s | Wall time |
| --- | --- | --- | --- | --- | --- | --- | --- |
| intervals | Qwen3-Coder 30B | True | True | 2 | 804 | 11.5 | 77.2 s |
| intervals | Qwen3.8 27B | True | True | 2 | 1280 | 3.8 | 535.6 s |
| ledger | Qwen3-Coder 30B | True | True | 2 | 484 | 11.9 | 44.5 s |
| ledger | Qwen3.8 27B | True | True | 2 | 551 | 3.2 | 340.3 s |
| tokens | Qwen3-Coder 30B | True | True | 2 | 666 | 11.6 | 60.3 s |
| tokens | Qwen3.8 27B | True | True | 2 | 1292 | 3.7 | 550.5 s |

Fixed prompts and budgets do not force equal reply lengths. Decode throughput divides reported output tokens by Ollama generation time; wall time also includes prompt processing and evaluation. Inspect both token volume and phase timing before attributing a wall-time difference to the model alone.

The accepted `intervals` and `tokens` documents were byte-identical to their initial drafts. The fixed polishing condition still requested a second pass. These wall times therefore include an extra revision/verification request even when it changed nothing; they are not optimized one-pass latency figures. First-call durations are retained in the raw evidence.

Candidate programs run in fresh Monty namespaces for each test case, with no host callbacks. Hickory reconstructs their exact output bytes. The polishing pass freezes the initial passing Python AST and reconstructs final source again. This preserves executable structure during polishing; it does not promise unchanged initial source bytes or independently prove explanation accuracy. Public checks guide repair; held-out cases are evaluated only after stopping.

- [intervals accepted document](../experiments/local-agent/2026-11-01-qwen38/intervals/project.md).
- [ledger accepted document](../experiments/local-agent/2026-11-01-qwen38/ledger/project.md).
- [tokens accepted document](../experiments/local-agent/2026-11-01-qwen38/tokens/project.md).

## Tool-and-submission fixture

One ledger run uses the existing action loop with read, candidate evaluation, status and literate-submission tools: focused instructions, 8192-token context, 768-token output cap, temperature 0.2 and a twelve-turn limit. This is an isolated development fixture, not the application approval flow.

| Model | Completed | Saved candidates | Held-out checks | Wall time |
| --- | --- | --- | --- | --- |
| Qwen3-Coder 30B, earlier stage trial | False | 0 | n/a | 259.2 s |
| Qwen3.8 27B | True | 1 | True | 572.8 s |

Actual handoffs recorded in the Qwen3.8 run: 1. Enabling handoff alone does not establish that one occurred. The earlier Qwen3-Coder stage trial failed before acceptance; its latency overlapped model loading and compilation. Timing is descriptive, not an isolated speedup estimate.

All seven saved candidate programs were separately reconstructed and regraded after the live trials. They passed their public and held-out cases in fresh interpreter namespaces, and the initial/final AST fingerprints match for each polishing pair.

The sample contains one run per task and model, and its simple functions do not establish long-horizon coding reliability, production tool use or parity with Fable 5. The document loop remains an isolated prototype. See the [earlier report](local-agent-results-2026-10-31.md) for the separate action-loop and context-placement limitations.

[Raw requests, replies, timings, candidates and receipts](../experiments/local-agent/2026-11-01-qwen38/records.json) include the download manifest, model digest and readiness probe. Reproduce with the [study guide](local-agent-study.md), substituting `--model qwen3.8:27b` and a new output directory.
