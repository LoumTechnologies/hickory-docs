---
id: TASK-57
title: 'LLM form UI: <hick:ask> XML format for structured user input from the LLM'
status: To Do
assignee: []
created_date: '2026-05-20 13:09'
labels:
  - ux
  - sdk
  - tui
  - web-ui
  - llm-forms
dependencies:
  - TASK-56
references:
  - docs/specs/user-input.md
  - hick-agent/crates/hick-agent-sdk/src/lib.rs
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Allow the LLM to request structured input from the user by embedding a `<hick:ask>` element in its response. The full format is specified in `docs/specs/user-input.md`. This task covers the parser, the mapping to `UserInputRequest`, and the rendering in both frontends.\n\n## Parser\n\nAdd a `FormParser` to `hick-agent-sdk` that:\n1. Scans each LLM response chunk (or complete response) for a `<hick:ask>` element.\n2. Extracts it and parses it into a `UserInputRequest` variant (see mapping below).\n3. Strips the `<hick:ask>` element from the displayed response text (it is replaced by the rendered form UI).\n4. Blocks the agent loop from proceeding until the `UserInputResponse` is received.\n\n## Mapping\n\n| `<hick:ask>` attributes | `UserInputRequest` variant |\n|---|---|\n| `type=\"choice\"` + `required=\"false\"` + `default` + `<hick:timeout>` | `Advisory` |\n| `type=\"choice\"` + `required=\"true\"` | `Required { kind: Choice(...) }` |\n| `type=\"text\"` | `Required { kind: FreeText }` |\n| `type=\"secret\"` | `Required { kind: Secret }` — value goes to SecretRegistry |\n| `type=\"form\"` | `Required` with multiple fields; secret fields go to SecretRegistry |\n\n## Free-form fallback\n\nEvery rendered form MUST include a \"Type a custom answer instead\" affordance. When the user activates it, the form controls are replaced with a plain text input. The free-form value is returned as `UserInputResponse::UserProvided(String)` exactly as if the user had typed it at the normal prompt.\n\n## TUI rendering\n\n- **Choice / Advisory**: overlay listing options with letter shortcuts (`1`, `2`, …) + countdown if advisory.\n- **Text / Secret**: overlay with a single input line (secret masked).\n- **Form**: multi-field overlay, Tab to move between fields, Enter to submit.\n- All overlays: `Ctrl-F` or `f` to switch to free-form text entry.\n\n## Web-UI rendering\n\n- Inline form card inserted into the conversation thread immediately after the LLM message that contained the `<hick:ask>`.\n- Choice: radio buttons or clickable chips.\n- Text: `<input type=\"text\">`.\n- Secret: `<input type=\"password\">`; value is NOT stored in React state after submission.\n- Form: all fields in a single card, Submit button.\n- Always visible: a \"Custom answer\" link that reveals a textarea.\n\n## Response serialisation\n\nThe structured response is returned to the LLM as a system message:\n```\n[User responded to form]\nrepo_name: my-project\nvisibility: private\ntoken: {{HICK_SECRET:token}}\n```\nSecret fields are represented as their placeholder. Non-secret fields are included as plaintext.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 FormParser detects <hick:ask> in LLM response and extracts it before display
- [ ] #2 Parsed form maps to the correct UserInputRequest variant per the table in the spec
- [ ] #3 Agent loop is blocked until UserInputResponse is received
- [ ] #4 <hick:ask> element is stripped from the rendered LLM message; the form UI appears in its place
- [ ] #5 Free-form fallback is available on every form type in both TUI and web-UI
- [ ] #6 Secret fields in <hick:ask type=secret> and <hick:field type=secret> write to SecretRegistry; value never appears in LLM context or session log
- [ ] #7 Advisory forms show the countdown and proceed with the stated default on timeout
- [ ] #8 Structured response sent to LLM uses placeholder for secret fields and plaintext for others
- [ ] #9 TUI keyboard shortcuts work: number keys for choices, Tab between form fields, Ctrl-F for free-form
- [ ] #10 Web-UI form card is inline in the conversation thread; custom answer textarea is accessible
<!-- AC:END -->
