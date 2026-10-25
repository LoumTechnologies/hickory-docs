<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep a prediction separate from the answer it is tested against: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

The shared prediction ledger records sealing, admission, execution outcome, exposure and promotion as distinct operations. Host adapters determine identities and keep private answers outside model-facing context. Native and browser adapters therefore need their own boundary witnesses in addition to ledger tests.

## Implementation

- [apps/agent/src/hickory_agent_worker/instructions.py](../../../shared/files/apps/agent/src/hickory_agent_worker/instructions.py.md)
- [apps/desktop/src-tauri/src/lib.rs](../../../shared/files/apps/desktop/src-tauri/src/lib.rs.md)
- [apps/desktop/src-tauri/src/server.rs](../../../shared/files/apps/desktop/src-tauri/src/server.rs.md)
- [apps/web/src/components/AgentChanges.tsx](../../../shared/files/apps/web/src/components/AgentChanges.tsx.md)
- [apps/web/src/components/AgentPicker.tsx](files/apps/web/src/components/AgentPicker.tsx.md)
- [apps/web/src/components/ChatDock.tsx](../../../shared/files/apps/web/src/components/ChatDock.tsx.md)
- [apps/web/src/components/FeatureSettings.tsx](files/apps/web/src/components/FeatureSettings.tsx.md)
- [apps/web/src/components/NativeWorkflowTour.tsx](files/apps/web/src/components/NativeWorkflowTour.tsx.md)
- [apps/web/src/components/PrivatePredictionCoach.tsx](../../../shared/files/apps/web/src/components/PrivatePredictionCoach.tsx.md)
- [apps/web/src/components/privatePredictionCoach.css](files/apps/web/src/components/privatePredictionCoach.css.md)
- [apps/web/src/lib/nativeWindowActions.ts](../../../shared/files/apps/web/src/lib/nativeWindowActions.ts.md)
- [apps/web/src/lib/uiActionBridge.tsx](../../../shared/files/apps/web/src/lib/uiActionBridge.tsx.md)
- [apps/web/src/lib/uiActions.ts](../../../shared/files/apps/web/src/lib/uiActions.ts.md)
- [apps/web/src/main.tsx](files/apps/web/src/main.tsx.md)
- [crates/hick-context-store/src/host.rs](../../../shared/files/crates/hick-context-store/src/host.rs.md)
- [crates/hick-dap/src/interactive_capture.rs](../../../support/files/crates/hick-dap/src/interactive_capture.rs.md)
- [crates/hick-lang-wasm/src/prediction.rs](../../../shared/files/crates/hick-lang-wasm/src/prediction.rs.md)
- [crates/hick-literate/examples/authorization_evidence.rs](files/crates/hick-literate/examples/authorization_evidence.rs.md)
- [crates/hick-literate/src/prediction.rs](files/crates/hick-literate/src/prediction.rs.md)
- [crates/hick-literate/src/prediction/combined.rs](files/crates/hick-literate/src/prediction/combined.rs.md)
- [crates/hick-literate/src/prediction/debugger.rs](files/crates/hick-literate/src/prediction/debugger.rs.md)
- [crates/hick-literate/src/prediction/debugger/tests.rs](files/crates/hick-literate/src/prediction/debugger/tests.rs.md)
- [crates/hick-literate/src/prediction/environment.rs](files/crates/hick-literate/src/prediction/environment.rs.md)
- [crates/hick-literate/src/prediction/gate.rs](../../../support/files/crates/hick-literate/src/prediction/gate.rs.md)
- [crates/hick-literate/src/prediction/interactive.rs](files/crates/hick-literate/src/prediction/interactive.rs.md)
- [crates/hick-literate/src/prediction/python_identity.py](files/crates/hick-literate/src/prediction/python_identity.py.md)
- [crates/hick-literate/src/prediction/source_question.rs](files/crates/hick-literate/src/prediction/source_question.rs.md)
- [crates/hick-observe/src/prediction.rs](../../../shared/files/crates/hick-observe/src/prediction.rs.md)
- [crates/hick-observe/src/prediction_publication.rs](files/crates/hick-observe/src/prediction_publication.rs.md)
- [crates/hick-observe/src/prediction_scope.rs](files/crates/hick-observe/src/prediction_scope.rs.md)
- [crates/hick-observe/src/prediction_types.rs](files/crates/hick-observe/src/prediction_types.rs.md)
- [crates/hickory-agent/src/lib.rs](files/crates/hickory-agent/src/lib.rs.md)
- [crates/hickory-agent/src/llm_openai.rs](../../../shared/files/crates/hickory-agent/src/llm_openai.rs.md)
- [crates/hickory-agent/src/provider.rs](../../../shared/files/crates/hickory-agent/src/provider.rs.md)
- [crates/hickory-agent/src/tools/dispatch.rs](files/crates/hickory-agent/src/tools/dispatch.rs.md)
- [crates/hickory-agent/src/tools/prediction.rs](files/crates/hickory-agent/src/tools/prediction.rs.md)
- [crates/hickory-agent/src/tools/preview.rs](../../../shared/files/crates/hickory-agent/src/tools/preview.rs.md)
- [crates/hickory-cli/examples/engine_client.rs](../../../shared/files/crates/hickory-cli/examples/engine_client.rs.md)
- [crates/hickory-cli/src/engine/client.rs](../../../shared/files/crates/hickory-cli/src/engine/client.rs.md)
- [crates/hickory-cli/src/engine/daemon.rs](../../../shared/files/crates/hickory-cli/src/engine/daemon.rs.md)
- [crates/hickory-cli/src/engine/mcp.rs](../../../shared/files/crates/hickory-cli/src/engine/mcp.rs.md)
- [crates/hickory-cli/src/engine/mod.rs](../../../shared/files/crates/hickory-cli/src/engine/mod.rs.md)
- [crates/hickory-cli/src/engine/private_policy.rs](files/crates/hickory-cli/src/engine/private_policy.rs.md)
- [crates/hickory-cli/src/engine/proxy.rs](../../../shared/files/crates/hickory-cli/src/engine/proxy.rs.md)
- [crates/hickory-cli/src/mcp_stdio.rs](files/crates/hickory-cli/src/mcp_stdio.rs.md)
- [crates/hickory-cli/src/python_agent.rs](../../../shared/files/crates/hickory-cli/src/python_agent.rs.md)
- [crates/hickory-cli/src/python_agent/host.rs](files/crates/hickory-cli/src/python_agent/host.rs.md)
- [crates/hickory-cli/src/python_agent/private.rs](files/crates/hickory-cli/src/python_agent/private.rs.md)
- [crates/hickory-cli/src/serve/acp/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/acp/mod.rs.md)
- [crates/hickory-cli/src/serve/agent.rs](../../../shared/files/crates/hickory-cli/src/serve/agent.rs.md)
- [crates/hickory-cli/src/serve/bootstrap.rs](../../../support/files/crates/hickory-cli/src/serve/bootstrap.rs.md)
- [crates/hickory-cli/src/serve/engineer_tour.rs](../../../shared/files/crates/hickory-cli/src/serve/engineer_tour.rs.md)
- [crates/hickory-cli/src/serve/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/mod.rs.md)
- [crates/hickory-cli/src/serve/private_practice.rs](files/crates/hickory-cli/src/serve/private_practice.rs.md)
- [crates/hickory-cli/src/serve/private_prediction_acp.rs](files/crates/hickory-cli/src/serve/private_prediction_acp.rs.md)
- [crates/hickory-cli/src/serve/private_prediction_agent.rs](files/crates/hickory-cli/src/serve/private_prediction_agent.rs.md)
- [crates/hickory-cli/src/serve/private_prediction_debugger.rs](files/crates/hickory-cli/src/serve/private_prediction_debugger.rs.md)
- [crates/hickory-cli/src/serve/private_prediction_recovery.rs](files/crates/hickory-cli/src/serve/private_prediction_recovery.rs.md)
- [crates/hickory-cli/src/serve/private_prediction_startup.rs](files/crates/hickory-cli/src/serve/private_prediction_startup.rs.md)
- [crates/hickory-cli/src/serve/private_predictions.rs](../../../shared/files/crates/hickory-cli/src/serve/private_predictions.rs.md)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../shared/files/crates/hickory-cli/src/serve/python_agent.rs.md)
- [crates/hickory-cli/src/serve/shell.rs](../../../shared/files/crates/hickory-cli/src/serve/shell.rs.md)
- [crates/hickory-cli/src/serve/socket.rs](../../../shared/files/crates/hickory-cli/src/serve/socket.rs.md)
- [scripts/test-private-prediction-ui.sh](../../../shared/files/scripts/test-private-prediction-ui.sh.md)

## Tests and fixtures

- [apps/agent/tests/test_worker.py](../../../shared/files/apps/agent/tests/test_worker.py.md)
- [apps/web/e2e/privateCoachLayout.spec.ts](../../../shared/files/apps/web/e2e/privateCoachLayout.spec.ts.md)
- [apps/web/e2e/privatePredictionCoach.spec.ts](../../../shared/files/apps/web/e2e/privatePredictionCoach.spec.ts.md)
- [apps/web/e2e/staticUiParity.spec.ts](../../../shared/files/apps/web/e2e/staticUiParity.spec.ts.md)
- [apps/web/src/components/AgentChanges.test.tsx](../../../shared/files/apps/web/src/components/AgentChanges.test.tsx.md)
- [apps/web/src/components/BrowserPredictionPractice.test.tsx](../../../shared/files/apps/web/src/components/BrowserPredictionPractice.test.tsx.md)
- [apps/web/src/components/FeatureSettings.test.tsx](../../../shared/files/apps/web/src/components/FeatureSettings.test.tsx.md)
- [apps/web/src/components/NativeWorkflowTour.test.tsx](../../../shared/files/apps/web/src/components/NativeWorkflowTour.test.tsx.md)
- [apps/web/src/components/PrivatePredictionCoach.test.tsx](../../../shared/files/apps/web/src/components/PrivatePredictionCoach.test.tsx.md)
- [apps/web/src/lib/nativeWindowActions.test.ts](../../../shared/files/apps/web/src/lib/nativeWindowActions.test.ts.md)
- [apps/web/src/lib/uiActionBridge.test.tsx](../../../shared/files/apps/web/src/lib/uiActionBridge.test.tsx.md)
- [apps/web/src/tour/browserPrediction.test.ts](../../../shared/files/apps/web/src/tour/browserPrediction.test.ts.md)
- [crates/hick-context-store/tests/lineage.rs](../../../shared/files/crates/hick-context-store/tests/lineage.rs.md)
- [crates/hick-dap/tests/private_adapter.rs](files/crates/hick-dap/tests/private_adapter.rs.md)
- [crates/hick-literate/tests/prediction_authorization.rs](files/crates/hick-literate/tests/prediction_authorization.rs.md)
- [crates/hick-literate/tests/prediction_combined.rs](files/crates/hick-literate/tests/prediction_combined.rs.md)
- [crates/hick-literate/tests/prediction_environment.rs](files/crates/hick-literate/tests/prediction_environment.rs.md)
- [crates/hick-literate/tests/prediction_output.rs](files/crates/hick-literate/tests/prediction_output.rs.md)
- [crates/hick-observe/src/prediction_scope_tests.rs](files/crates/hick-observe/src/prediction_scope_tests.rs.md)
- [crates/hick-observe/src/prediction_tests.rs](../../../shared/files/crates/hick-observe/src/prediction_tests.rs.md)
- [crates/hickory-agent/tests/prediction_verification.rs](files/crates/hickory-agent/tests/prediction_verification.rs.md)
- [crates/hickory-cli/src/engine/client_prediction_tests.rs](../../../shared/files/crates/hickory-cli/src/engine/client_prediction_tests.rs.md)
- [crates/hickory-cli/src/serve/private_prediction_acp_tests.rs](files/crates/hickory-cli/src/serve/private_prediction_acp_tests.rs.md)
- [crates/hickory-cli/src/serve/private_prediction_tests.rs](../../../shared/files/crates/hickory-cli/src/serve/private_prediction_tests.rs.md)
- [crates/hickory-cli/tests/prediction_engine.rs](files/crates/hickory-cli/tests/prediction_engine.rs.md)
- [crates/hickory-cli/tests/prediction_http.rs](files/crates/hickory-cli/tests/prediction_http.rs.md)

## Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
