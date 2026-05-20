---
id: TASK-45
title: 'Multi-machine eval config: per-machine capability declarations in agent TOML'
status: To Do
assignee: []
created_date: '2026-05-20 12:16'
labels:
  - eval
  - config
  - local-model
dependencies:
  - TASK-44
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a `[machine]` section to the agent config TOML that declares what this machine can run: available backends (Metal/CUDA/CPU), max model size in GB, GGUF model paths, and which eval config combinations this machine should attempt. The eval coordinator reads this section to schedule only feasible combinations on each machine. A 2018 MacBook Pro with weak Metal should be declared as CPU-primary with small model paths; a machine with a discrete GPU declares its VRAM. This prevents the eval coordinator from scheduling a 70B model on a machine with 8 GB RAM.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 [machine] section is documented with: backends (list), max_model_gb (float), model_paths (list of {name, path, size_gb}), eval_configs (list of config names to evaluate)
- [ ] #2 Eval coordinator skips combinations whose model size_gb exceeds the machine's max_model_gb
- [ ] #3 Config validation emits a clear error if a declared model_path does not exist
- [ ] #4 Two machines with different [machine] sections pull from the same repo and each only run feasible combinations
- [ ] #5 MacBook Pro (CPU/Metal, limited VRAM) and GPU desktop configs are documented as examples in configs/
<!-- AC:END -->
