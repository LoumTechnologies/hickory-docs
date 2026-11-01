# Does placing evidence beside source help the local model?

For Hickory maintainers deciding whether to enable interleaved agent context by default. This pilot measures association of captured observations with opaque action handles, not completion of a debugging session.

On Qwen3.8 27B, interleaving changed correct selections by **+16.7 percentage points** in this small fixed sample. This is a descriptive result from six paired tasks in two generated scene seeds. It does not establish a population effect or justify a general speedup claim.

## Controlled comparison

Each scene contains four source locations, current and historical observations, and enabled and disabled action handles. Tasks select Run for the current failing cell, Step over for the paused frame with local `n = 0`, or Open profile for the largest current timing. Both conditions receive exactly the same source and context blocks, once each, with identical total UTF-8 bytes. Only adjacency changes. Actual saved requests were checked for this equality before producing this report.

The separate layout lists sources followed by shuffled observations/actions. The interleaved layout moves each location's observations and actions beside its source block, retaining their relative order. Both are structured JSON blocks; this does not test polished Markdown presentation or the full Hickory editor projection. Historical observations and disabled controls remain visible in both conditions.

Settings: `qwen3.8:27b`, CPU only, six threads, 8192 context tokens, 96 output tokens, temperature 0.2, top_p 0.95, top_k 20, model sampling seed 42, thinking disabled. Scene seeds 0 and 1. Separate runs first for seed 0; interleaved runs first for seed 1. The exact model digest, quantization, Ollama version, prompts, responses and measured phases are retained below.

| Measure | Separate | Interleaved |
| --- | ---: | ---: |
| Correct action | 5/6 | 6/6 |
| Wrong source, enabled handle | 1 | 0 |
| Disabled/historical handle | 0 | 0 |
| Invalid/fabricated handle | 0 | 0 |
| Transport errors | 0 | 0 |
| Output cutoffs | 0 | 0 |
| Median reported input tokens | 1614 | 1614 |
| Median output tokens | 39.5 | 39.5 |
| Median wall time | 190.0 s | 188.8 s |
| Median prompt + generation time | 188.7 s | 188.5 s |

Paired outcomes: both correct 5; separate only 0; interleaved only 1; neither 0.

| Task | Seed | Separate correct | Interleaved correct |
| --- | ---: | --- | --- |
| debug | 0 | True | True |
| debug | 1 | True | True |
| profile | 0 | True | True |
| profile | 1 | False | True |
| run | 0 | True | True |
| run | 1 | True | True |

Target locations are also unbalanced in this small generated sample: {'unit_3.py': 5, 'unit_0.py': 1}. Source order is fixed, so source-position heuristics are another reason not to generalize. A larger study should balance target position as well as condition order.

Wall time includes model loading and runtime overhead. The first call starts with an unloaded model; recorded phase timings separate loading from inference. Alternating order reduces a simple always-first bias but does not eliminate prompt-cache or order effects. Prompt token counts are reported by Ollama, whose cache behavior also affects phase durations. No controlled application-latency improvement is established.

## What these numbers support

The earlier Qwen3-Coder 30B pilot at eight locations and four scene seeds scored 6/12 separate versus 2/12 interleaved. The current study has fewer locations, fewer scene seeds and a different model; the two studies must not be pooled or interpreted as a controlled model ranking. The two pilots do not show a consistent advantage for interleaving. Keep the production default structured pending realistic task evidence.

Both conditions have synthetic run/debug/profile observations and action descriptors. This isolates their joint placement, not the value of providing those capabilities at all, and not the individual contribution of log placement versus button placement. All observations are synthetic captures; selected handles are scored without invoking a live registry. The safety checks that real actions require are separately implemented in `UiActions` and are not validated by this model-selection pilot.

In the current application, source-bound captured observations can be rendered beside matching lines when their URI/buffer and revision agree. Many action descriptors still appear in separate observation groups. The pilot is not evidence that every debugger or run control already has a source association, or that the experimental renderer accelerates real edits.

A stronger follow-up should use held-out reproducible bugs and real command receipts. Cross log placement (inline/separate) with action placement (inline/separate), holding facts, available actions, permissions, model and budgets fixed; include independent scene seeds and balanced condition order. Separately remove logs or actions to measure capability availability. Grade verified task completion, stale/wrong-target actions, model calls, tokens and elapsed time. Report failure and interruption rates alongside successes. This would separate association benefits from information availability and presentation quality.

Reproduce this pilot:

```sh
PYTHONPATH=apps/agent/src apps/agent/.venv/bin/python -m hickory_agent_worker.context_study \
  --model qwen3.8:27b --seeds 2 --cells 4 \
  --output target/local-agent/new-placement-run
```

[Exact requests, responses, identity and phase timings](../experiments/local-agent/2026-11-01-placement/records.json). [Study design and implementation limits](local-agent-study.md). [Earlier pilot](local-agent-results-2026-10-31.md).
