---
id: TASK-23
title: Agent A/B Evaluation Framework
status: To Do
assignee: []
created_date: '2026-05-16 12:12'
labels:
  - evaluation
  - a-b-testing
  - agent
milestone: m-2
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Build a system for measuring and comparing AI coding agent configurations on quality and cost so we can data-drive the selection of the winning combination for public release.

## Context

The agent lives under `hick-agent/crates/`. Architecture has three layers (see CLAUDE.md):
- **hick-agent-sdk**: core runtime, no hick pipeline knowledge
- **hick-agent-* extension crates**: optional capabilities (e.g. script-first, hick-edit)
- **Binary crates** (adjective-noun names): each wires a fixed set of extensions via explicit `Cargo.toml` deps — no hidden feature gates

Each binary is a candidate in A/B evaluation. The winning combination becomes the public release.

## Combinatorial explosion problem

As new extension crates are added the number of possible combinations grows as 2^n. The evaluation system MUST NOT assume a small fixed set of combinations. Design to handle:
- Declarative combination specs (which extensions are active) rather than requiring a compiled binary per combination
- Smart sampling strategies (fractional factorial design, multi-armed bandit, or similar) rather than exhaustive grid search
- Per-extension attribution: which individual extensions drive quality up or cost down

## What "cost" means here

Not all tokens are priced the same. A run's true cost depends on:
- Which model was used (Opus, Sonnet, Haiku — each has different per-token pricing)
- Token type: cache-read input, cache-write input, uncached input, output, thinking (extended-thinking mode)
- Number of API calls (turn count matters for multi-turn sessions)

## What "quality" means here

Quality is task-specific and needs automated checks:
- Does the code compile?
- Do tests pass?
- Does a task-supplied acceptance script pass?
- (Optional) LLM-judge score for subjective quality

## Subtasks

- TASK-X.1 Benchmark corpus — task format spec + initial suite
- TASK-X.2 Token cost model — Anthropic pricing tables + cost computation crate
- TASK-X.3 Eval runner — run one agent config against one task, emit XML result
- TASK-X.4 Quality scorer — automated acceptance checking
- TASK-X.5 Eval coordinator — batch runner with smart sampling across the combination space
- TASK-X.6 Results reporting — comparison tables, per-extension attribution, Pareto frontier
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Running `just eval` against two or more agent configurations produces a ranked comparison report showing quality score and true token cost per configuration.
- [ ] #2 The system handles at least 5 extensions without requiring exhaustive 2^5 = 32 runs to produce a useful ranking.
- [ ] #3 Cost computation correctly distinguishes cache-read, cache-write, uncached-input, output, and thinking tokens using current Anthropic pricing.
- [ ] #4 All result artifacts are stored as XML (not JSON) for LLM parseability.
- [ ] #5 A new extension can be added to the evaluation matrix without changes to the runner or reporter — only a config update is needed.
<!-- AC:END -->
