# Run a private prediction practice through ACP

Configure an installed ACP adapter in ordinary Settings → Agents before creating
the private practice. Give it a separate `private_prediction_args` array; `[]`
is an explicit choice. Ordinary `args` are never reused. Select arguments that
start fresh and do not load ordinary conversations or configuration. Leave
`workspace_filesystem` false. The adapter must support ACP v1 and HTTP MCP. If it requires an explicit
authentication request, set `private_prediction_auth_method` to an advertised
method id. For the pinned Codex ACP adapter, use `"api-key"` with an environment
`OPENAI_API_KEY` or `CODEX_API_KEY`. This request precedes the fresh session; it
does not copy ordinary sign-in files.

In the private coach, clear Provider and Model, enter the configured adapter id
in **ACP adapter (optional)**, then Delegate. Leaving it blank uses confined Python.
ACP receives environment API credentials, a temporary home/config/data/cache and
empty working directory. Hickory does not copy ordinary sign-in files, editor
context, conversation history, provider base URLs or engine tokens. Configure
credentials for the adapter through its supported environment API key mechanism;
Hickory's saved Python provider selection is independent.

The managed ACP client refuses filesystem, terminal, permission and extension
requests. Its private MCP server supplies approved source, mandatory exact-edit
review, sealed verification and retained-stop ordering through the existing gate.
Optional UI tools attach only to the originating private window. A completed ACP
turn must pass the host repair-and-verify completion gate. Model notifications
and raw adapter diagnostics are not used as verification or copied to the coach.
Stop cancels pending reviews and retires the tool context; already dispatched
operations may have effects. End retains the same private history protections.

This boundary governs Hickory's supplied context and managed interfaces. It does
not sandbox the external executable's own OS access or prove that an arbitrary
adapter honors its fresh-context arguments. Treat the executable and separate
private argument profile as trusted configuration. A provider workflow and native
release still require their own witnesses.

For focused scripted acceptance without a paid provider:

```sh
scripts/test-private-prediction-ui.sh --acp-fixture
```

This runs the actual engine, ACP process, private MCP tools, browser, execution and
DAP comparisons. Predictions, the repair and review decisions are scripted. It is
not evidence of model reasoning, human review or an accepted publication.
