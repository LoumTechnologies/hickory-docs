# hick-policy

Security policy engine that evaluates data-flow and data-access decisions by
delegating to an external Python subprocess.

`PolicyEvaluator` serializes `FlowPolicyInput` or `AccessPolicyInput` structs
to JSON on stdin, spawns a configurable Python function (path passed as
`sys.argv[1]`, never interpolated into code), and deserializes the
`PolicyDecision` result. A 5-second timeout is enforced.

## Policy decisions

- `Allow`
- `Deny { reason }` 
- `Prompt { reason }` — escalate for human review

## Auxiliary modules

- `generator` — `SuggestionGenerator` produces candidate policy rules from
  observed access patterns
- `suggestion` — `PolicySuggestion` types for ranking and presenting candidates
