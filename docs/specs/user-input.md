# User Input Specification

## Scope

This document specifies the three categories of user input request the agent
runtime MUST support, their timeout and fallback semantics, frontend rendering
requirements, and the `<hick:ask>` XML format the LLM uses to request
structured input from the user.

---

## Input Request Taxonomy

All user input requests MUST be represented as `UserInputRequest` in
`hick-agent-sdk`. There are exactly three variants. Implementations MUST NOT
conflate them or use one variant where another applies.

### 1. Advisory

The agent has a reasonable default and MAY proceed without a response. Asking
the user improves quality or avoids wasted work; silence costs tokens at worst.

```rust
Advisory {
    question: String,
    default: String,       // what the agent will do if the user does not respond
    timeout: Duration,     // after which the agent proceeds with `default`
}
```

**Timeout**: Advisory prompts MUST carry a timeout. When the timeout expires,
the agent MUST proceed with the stated `default` and MUST log the
auto-resolution. The `default` value MUST be displayed in the UI before the
timeout fires so the user knows what will happen if they do not respond.

**AFK**: the agent MUST self-resolve using `default` without user interaction.
The AFK activity feed MUST record a dim entry: `[auto] Used default: <default>`.

### 2. Required

The agent is genuinely blocked. There is no valid default the runtime can
supply — the answer MUST come from the user.

```rust
Required {
    prompt: String,
    kind: RequiredKind,    // FreeText | Secret | Choice(Vec<String>)
    // no timeout field — there is no fallback value to apply
}
```

**Timeout**: Required prompts MUST NOT carry a timeout. The runtime MUST
remain blocked until the user responds or the session is terminated. Any
implementation that applies a timeout or proceeds with an assumed value for a
`Required` prompt is non-conformant.

**AFK**: the AFK modal MUST display a high-priority "agent is waiting" card
that makes clear the agent has halted. This card MUST NOT carry a countdown.
The agent MUST NOT proceed until the user responds, even if AFK mode is exited
and re-entered.

**`RequiredKind` values**:

| Variant | MUST render as |
|---------|----------------|
| `FreeText` | Single-line or multi-line text input |
| `Secret` | Masked input; value MUST NOT be echoed, logged, or sent to the LLM |
| `Choice(Vec<String>)` | Selectable list; free-form text entry MUST always be available as an alternative |

### 3. SecurityApproval

A specific action is pending that the security policy has classified as
`requires_approval`. The user MUST explicitly authorise or deny it.

```rust
SecurityApproval {
    action: PendingAction,   // NetworkRequest | ToolCall
    timeout: Duration,       // per security-prompts spec; timeout MUST result in Deny
}
```

All timeout, activation-delay, logging, and UI requirements for
`SecurityApproval` are specified in `docs/specs/security-prompts.md`.

---

## LLM Form Format (`<hick:ask>`)

The LLM MAY embed a `<hick:ask>` element anywhere in its response to request
structured input from the user. The agent runtime MUST detect, extract, and
render the form before the response is shown to the user. The LLM MUST NOT
continue past the `<hick:ask>` in the same turn — it MUST await the user's
reply before proceeding.

The user MUST always be able to provide a free-form text answer in place of
the structured controls, regardless of form type. Frontends MUST expose a
visible "Type a custom answer" affordance at all times on every form.

### Simple choice (maps to `Advisory` or `Required`)

```xml
<hick:ask type="choice" required="false" default="2-space">
  <hick:question>How many spaces for indentation?</hick:question>
  <hick:option value="2-space">2 spaces</hick:option>
  <hick:option value="4-space">4 spaces</hick:option>
  <hick:option value="tab">Tabs</hick:option>
  <hick:timeout>30</hick:timeout>
</hick:ask>
```

- `required="false"` with a `default` attribute maps to `Advisory`. The
  `<hick:timeout>` element (value in seconds) MUST be present and MUST be ≥ 5.
- `required="true"` with no `default` maps to `Required { kind: Choice(...) }`.
  `<hick:timeout>` MUST be omitted. Its presence MUST be treated as a
  parse error.

### Text field (maps to `Required { kind: FreeText }`)

```xml
<hick:ask type="text">
  <hick:question>What is your GitHub username?</hick:question>
</hick:ask>
```

### Secret field (maps to `Required { kind: Secret }`)

```xml
<hick:ask type="secret">
  <hick:question>Personal access token (will not be sent to the LLM)</hick:question>
</hick:ask>
```

The runtime MUST NOT include the user's answer to a `type="secret"` field in
any subsequent LLM message. The value MUST be written to the session-scoped
`SecretRegistry` and referenced thereafter only by its placeholder. See
`docs/specs/secret-scanning.md`.

### Multi-field form (maps to `Required` with multiple fields)

```xml
<hick:ask type="form">
  <hick:question>Repository details</hick:question>
  <hick:field name="repo_name" type="text" required="true">Repository name</hick:field>
  <hick:field name="visibility" type="choice" required="true">
    Visibility
    <hick:option value="public">Public</hick:option>
    <hick:option value="private">Private</hick:option>
  </hick:field>
  <hick:field name="token" type="secret" required="false">
    Personal access token (optional)
  </hick:field>
</hick:ask>
```

The runtime MUST collect all fields before returning a response to the LLM.
Non-secret field values MUST be included in the reply as plaintext. Secret
field values MUST be stored in the `SecretRegistry`; the LLM MUST receive only
their placeholder names.

### `<hick:ask>` parsing rules

- The runtime MUST strip the `<hick:ask>` element from the text shown to the
  user and MUST replace it with the rendered form UI.
- A `<hick:timeout>` element inside a `required="true"` form MUST be rejected
  at parse time with a descriptive error surfaced to the user.
- An `<hick:ask>` element with an unrecognised `type` attribute MUST be
  treated as `type="text"` and MUST log a warning.
- Multiple `<hick:ask>` elements in a single LLM response MUST be presented
  sequentially; the agent loop MUST NOT proceed to the second until the first
  is resolved.

### Rendering requirements

| Frontend | Advisory | Required / Form |
|----------|----------|-----------------|
| TUI | Overlay with options + timeout countdown | Blocking overlay, no countdown |
| Web-UI | Inline card with options + timeout countdown | Inline card, no countdown |

- The "Type a custom answer" affordance MUST be visible at all times.
- Advisory forms MUST display the `default` value and the remaining timeout
  prominently.
- `type="secret"` fields MUST mask input. The value MUST NOT appear in the
  conversation log or XML session log; implementations MUST substitute
  `[secret provided]` in any log entry that would otherwise contain the value.

### Response serialisation

The structured response MUST be returned to the LLM as a system turn in the
following format:

```
[User responded to form]
repo_name: my-project
visibility: private
token: {{HICK_SECRET:token}}
```

Secret fields MUST be represented as their `SecretRegistry` placeholder.
Non-secret fields MUST be included as plaintext.
