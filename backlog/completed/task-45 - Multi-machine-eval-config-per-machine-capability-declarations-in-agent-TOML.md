---
id: TASK-45
title: 'Multi-machine eval config: per-machine capability declarations in agent TOML'
status: Done
assignee: []
created_date: '2026-05-20 12:16'
updated_date: '2026-05-20 17:37'
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
- [x] #1 [machine] section is documented with: backends (list), max_model_gb (float), model_paths (list of {name, path, size_gb}), eval_configs (list of config names to evaluate)
- [x] #2 Eval coordinator skips combinations whose model size_gb exceeds the machine's max_model_gb
- [x] #3 Config validation emits a clear error if a declared model_path does not exist
- [x] #4 Two machines with different [machine] sections pull from the same repo and each only run feasible combinations
- [x] #5 MacBook Pro (CPU/Metal, limited VRAM) and GPU desktop configs are documented as examples in configs/
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Replaced the empty `MachineSection` stub with a full implementation in `hick-agent/crates/hick-agent/src/config.rs`:

- `ModelPathEntry` struct: `name`, `path` (PathBuf), `size_gb` (f64)
- `MachineSection`: `backends` (Vec<String>), `max_model_gb` (Option<f64>), `model_paths` (Vec<ModelPathEntry>), `eval_configs` (Vec<String>) — all `#[serde(default)]` so existing empty `[machine]` sections parse unchanged
- `can_run_model(size_gb)` predicate: `true` when no limit is declared, otherwise `size_gb <= max_model_gb`
- `validate_paths(base_dir)` called from `load_from_file` — errors with the entry name if a declared model file is absent; resolves relative paths against the config file's parent directory
- 10 new unit tests covering parse, defaults, feasibility predicate (true/false/no-limit), missing path error, existing path OK, relative path resolution
- Two example configs: `configs/macbook-2018.toml` (CPU/Metal, max 8 GB) and `configs/gpu-desktop.toml` (CUDA, max 40 GB)
- Updated `standard.toml` and `hick-edit.toml` comment blocks to document the new fields
- All workspace tests pass (0 failures)
<!-- SECTION:FINAL_SUMMARY:END -->
