# Local models on historical nloum code — 1 November 2026

For the author comparing local-model task completion with retained pre-2021 solutions. This is an initial executable dataset, not a measurement of overall programmer ability.

Eight tasks come from CodeIO, PackageProjectDependencySwitcher and letter-classifier, at four source commits dated 2019–2020. Both author and committer dates precede 2021. The set contains two repair methods from one commit, one exception-factory refactor and five method completions. Discovery inventoried 63 commits across five cloned repositories; exclusions and source blobs are retained. The empty Replace-Package repository was discovered but not cloned.

All eight historical references passed the public and held-out checks, and all eight broken/missing baselines failed the public check. This is how cases were qualified; it is not an unbiased human success rate. Methods use their original text with newline normalization in small explicit fixtures. The original WPF, NuGet and Roslyn projects were not restored or tested as full applications.

## Fixed-budget results

| Model/reference | Public passed | Held-out passed | First-attempt held-out passed | Model calls | Total task time |
| --- | ---: | ---: | ---: | ---: | ---: |
| Qualified historical methods | 8/8 | 8/8 | n/a | n/a | Human work time unavailable |
| qwen3-coder:30b | 7/8 | 7/8 | 7/8 | 9 | 9.1 min |
| qwen3.8:27b | 6/8 | 6/8 | 6/8 | 10 | 45.5 min |

| Case | Historical reference | Qwen3-Coder 30B | Qwen3.8 27B |
| --- | --- | --- | --- |
| [codeio-exception-factory](../experiments/historical-code/2026-11-01/cases/codeio-exception-factory.md) | pass | pass, 1 attempt(s), 85.9 s | pass, 1 attempt(s), 158.8 s |
| [codeio-strip-type-arguments](../experiments/historical-code/2026-11-01/cases/codeio-strip-type-arguments.md) | pass | pass, 1 attempt(s), 24.4 s | pass, 1 attempt(s), 85.2 s |
| [codeio-variable-name](../experiments/historical-code/2026-11-01/cases/codeio-variable-name.md) | pass | pass, 1 attempt(s), 36.2 s | pass, 1 attempt(s), 103.1 s |
| [codeio-cartesian-product](../experiments/historical-code/2026-11-01/cases/codeio-cartesian-product.md) | pass | fail, 2 attempt(s), 171.3 s | fail, 2 attempt(s), 691.9 s |
| [letters-next-image](../experiments/historical-code/2026-11-01/cases/letters-next-image.md) | pass | pass, 1 attempt(s), 35.8 s | pass, 1 attempt(s), 122.4 s |
| [letters-record-label](../experiments/historical-code/2026-11-01/cases/letters-record-label.md) | pass | pass, 1 attempt(s), 32.3 s | pass, 1 attempt(s), 149.9 s |
| [dependencies-to-project](../experiments/historical-code/2026-11-01/cases/dependencies-to-project.md) | pass | pass, 1 attempt(s), 81.2 s | fail, 2 attempt(s), 966.0 s |
| [dependencies-to-package](../experiments/historical-code/2026-11-01/cases/dependencies-to-package.md) | pass | pass, 1 attempt(s), 77.6 s | pass, 1 attempt(s), 455.4 s |

The method-only response contract and reduced wrapper are part of this benchmark interface; a full-class reply may fail compilation even when some algorithmic content is plausible. The score requires public success and held-out success. A failed score includes attempts that never reached held-out evaluation because public checks or response format failed. A model may implement a different algorithm and pass; textual similarity is not the score. At most two attempts are allowed, with public feedback only. Repair stops at public success and held-out checks run afterward, without another opportunity to learn from them. Failed, malformed and truncated attempts remain in the raw records.

| Model | Reported input tokens | Output tokens | Cutoffs | Request/format errors | Public compile failures | Public execution failures | Reported model loading |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| qwen3-coder:30b | 5183 | 4271 | 0 | 1 | 0 | 1 | 52.8 s |
| qwen3.8:27b | 7916 | 5772 | 1 | 1 | 2 | 1 | 53.1 s |

| Model | Prompt processing | Generation | Model loading | Native compilation | Native check execution |
| --- | ---: | ---: | ---: | ---: | ---: |
| qwen3-coder:30b | 71.2 s | 366.7 s | 52.8 s | 48.3 s | 4.7 s |
| qwen3.8:27b | 903.5 s | 1721.4 s | 53.1 s | 47.5 s | 4.2 s |

Both models use CPU-only inference on the 32 GiB Ryzen 5 3600 machine, six threads, 8192-token context, 1536-token output cap, temperature 0.2, top_p 0.95, top_k 20, seed 42 and thinking disabled. They run sequentially and unload between models. Task time includes inference and native grading; the cold first call includes model loading. Fixed settings are not per-model sampling optimization. Host load, cache state and thermal behavior are not controlled, so timing is descriptive. Qualification time and the pause between models are excluded from summed task time. The first model transition was refused by the memory guard before any Qwen3.8 request; after memory reclamation, the identical configuration resumed and reused completed task records. The interruption is retained in the raw bundle.

## What this says about comparison with the author

The benchmark asks whether models can recover the specified behavior of methods that the author had committed years earlier. It cannot reconstruct the original difficulty, prior knowledge, time spent, false starts or assistance. The specifications and checks were created now after inspecting the solutions. They give the models an explicit behavior contract and a small source projection; original problem discovery, repository navigation and requirements inference are not measured. Some collaborators are reduced test doubles; the variable-name helper substitutes a later pre-2021 implementation for Humanizer. Those choices are recorded per case.

Eight method tasks share four source commits and several helpers. This is a small curated set, not eight independent samples of general engineering ability. Passing finite checks is not proof of complete correctness. Public pre-2021 code may have appeared in training data, and the cutoff does not establish contamination freedom. Further optimization on these cases would make them development data; a held-out set of new commits is needed for an unbiased later comparison.

## Readable context and debugger follow-up

All requests use prose, fenced C# and plain-text public-check feedback immediately beside the current method. JSON is an artifact format rather than the model-facing document. The target historical solution, commit messages and held-out checks are outside the model request. Some supplied collaborator methods are targets in other cases; this overlap is another reason not to treat the tasks as independent. The runner never compiles held-out check bodies into public evaluations. The focused safety witnesses check those information boundaries, stale reference rejection and cutoff refusal.

This run does not compare layouts. It also does not exercise live debugger stepping. The [benchmark guide](historical-code-benchmark.md) records a separate design for bounded debugger probes: evaluate expressions, step until a declared condition or stop limit, and return a source-aligned trace in one model interaction. Existing UI step actions acknowledge dispatch before a stop, while the native bridge waits for the settled result. A correct batch must wait, refresh stop/frame scope, retain existing permissions and honor actual reverse-execution capabilities.

[Dataset and reference code](../experiments/historical-code/2026-11-01/dataset.json), [provenance and exclusions](../experiments/historical-code/2026-11-01/provenance.json), [native reference witnesses](../experiments/historical-code/2026-11-01/references.json), and [exact model requests, replies, candidates and timings](../experiments/historical-code/2026-11-01/run-records.json). Reproduction commands and scope are in the [guide](historical-code-benchmark.md).

## Post-hoc concrete-feedback development probe

The Qwen3-Coder Cartesian-product failure was deliberately selected after the primary result was known. Its first candidate and original second-turn instructions were reused. An alternate public diagnostic driver reported the actual and expected sequence instead of only `Assertion failed`. The repair was graded with the original frozen public checks; held-out checks would run only after public success. This is a single-case development intervention, not an unbiased held-out result or a debugger-batching test.

Public diagnostic supplied to the alternate repair request:

```text
Exception: Expected sequence: 1,3 -> 2,3 -> 1,4 -> 2,4; actual sequence: 1,3 -> 1,4 -> 2,3 -> 2,4
```

The extra repair call: public passed **False**, held-out **not run (public check failed)**, elapsed 191.6 seconds. The original second attempt failed the single-block response contract. Neither the exploratory call nor its outcome changes the primary table. Exact trace diagnostics, requests, replies, configuration and receipts are included in the raw bundle.

For diagnosis, the 2 original second-response code blocks were separately compiled and checked after the primary trial: 0 passed the public check. This supplemental evaluation does not change the original response-format failure or primary scores.

A separate interface diagnosis placed Qwen3.8’s unmodified full-class response at namespace scope and exposed its method to the original test driver with `using static Combinatorics`. In that compatible wrapper, public passed **True** and held-out passed **True**. This requires no further model call and does not repair or change the algorithm. It is not the primary method-only interface, so it does not retroactively alter the main score.
