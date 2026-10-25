<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli: files 1–50

For: maintainers editing this component or tracing its affected guarantees.

Open a construction below to edit its bytes and inspect the related contracts.

- [crates/hickory-cli/Cargo.toml](files/crates/hickory-cli/Cargo.toml.md)
- [crates/hickory-cli/build.rs](files/crates/hickory-cli/build.rs.md)
- [crates/hickory-cli/examples/acp_fixture.rs](files/crates/hickory-cli/examples/acp_fixture.rs.md)
- [crates/hickory-cli/examples/fskit_bundle.rs](files/crates/hickory-cli/examples/fskit_bundle.rs.md)
- [crates/hickory-cli/examples/fskit_live.rs](files/crates/hickory-cli/examples/fskit_live.rs.md)
- [crates/hickory-cli/examples/python_agent_ui_fixture.rs](files/crates/hickory-cli/examples/python_agent_ui_fixture.rs.md)
- [crates/hickory-cli/src/agent_command.rs](files/crates/hickory-cli/src/agent_command.rs.md)
- [crates/hickory-cli/src/code_model.rs](files/crates/hickory-cli/src/code_model.rs.md)
- [crates/hickory-cli/src/coverage/commands.rs](files/crates/hickory-cli/src/coverage/commands.rs.md)
- [crates/hickory-cli/src/coverage/mapping.rs](files/crates/hickory-cli/src/coverage/mapping.rs.md)
- [crates/hickory-cli/src/coverage/mod.rs](files/crates/hickory-cli/src/coverage/mod.rs.md)
- [crates/hickory-cli/src/coverage/real_tests.rs](files/crates/hickory-cli/src/coverage/real_tests.rs.md)
- [crates/hickory-cli/src/coverage/results.rs](files/crates/hickory-cli/src/coverage/results.rs.md)
- [crates/hickory-cli/src/coverage/runner.rs](files/crates/hickory-cli/src/coverage/runner.rs.md)
- [crates/hickory-cli/src/coverage/store.rs](files/crates/hickory-cli/src/coverage/store.rs.md)
- [crates/hickory-cli/src/diagnostics.rs](files/crates/hickory-cli/src/diagnostics.rs.md)
- [crates/hickory-cli/src/doc_cli_args.rs](files/crates/hickory-cli/src/doc_cli_args.rs.md)
- [crates/hickory-cli/src/engine/client/lifecycle.rs](files/crates/hickory-cli/src/engine/client/lifecycle.rs.md)
- [crates/hickory-cli/src/engine/watch.rs](files/crates/hickory-cli/src/engine/watch.rs.md)
- [crates/hickory-cli/src/engine/writes.rs](files/crates/hickory-cli/src/engine/writes.rs.md)
- [crates/hickory-cli/src/failed_run.rs](files/crates/hickory-cli/src/failed_run.rs.md)
- [crates/hickory-cli/src/ingest/scratchpad.rs](files/crates/hickory-cli/src/ingest/scratchpad.rs.md)
- [crates/hickory-cli/src/ingest/tests.rs](files/crates/hickory-cli/src/ingest/tests.rs.md)
- [crates/hickory-cli/src/init_hook.sh](files/crates/hickory-cli/src/init_hook.sh.md)
- [crates/hickory-cli/src/language_commands.rs](files/crates/hickory-cli/src/language_commands.rs.md)
- [crates/hickory-cli/src/main_thread.rs](files/crates/hickory-cli/src/main_thread.rs.md)
- [crates/hickory-cli/src/mcp/tests/tests.rs](files/crates/hickory-cli/src/mcp/tests/tests.rs.md)
- [crates/hickory-cli/src/mcp_catalogue.rs](files/crates/hickory-cli/src/mcp_catalogue.rs.md)
- [crates/hickory-cli/src/mcp_preview.rs](files/crates/hickory-cli/src/mcp_preview.rs.md)
- [crates/hickory-cli/src/open_app.rs](files/crates/hickory-cli/src/open_app.rs.md)
- [crates/hickory-cli/src/output.rs](files/crates/hickory-cli/src/output.rs.md)
- [crates/hickory-cli/src/output_format.rs](files/crates/hickory-cli/src/output_format.rs.md)
- [crates/hickory-cli/src/performance/browser.rs](files/crates/hickory-cli/src/performance/browser.rs.md)
- [crates/hickory-cli/src/performance/mod.rs](files/crates/hickory-cli/src/performance/mod.rs.md)
- [crates/hickory-cli/src/performance/native.rs](files/crates/hickory-cli/src/performance/native.rs.md)
- [crates/hickory-cli/src/performance/real_tests.rs](files/crates/hickory-cli/src/performance/real_tests.rs.md)
- [crates/hickory-cli/src/performance/resolve.rs](files/crates/hickory-cli/src/performance/resolve.rs.md)
- [crates/hickory-cli/src/performance/store.rs](files/crates/hickory-cli/src/performance/store.rs.md)
- [crates/hickory-cli/src/project_command.rs](files/crates/hickory-cli/src/project_command.rs.md)
- [crates/hickory-cli/src/project_index.rs](files/crates/hickory-cli/src/project_index.rs.md)
- [crates/hickory-cli/src/python_agent_main.rs](files/crates/hickory-cli/src/python_agent_main.rs.md)
- [crates/hickory-cli/src/selected_run.rs](files/crates/hickory-cli/src/selected_run.rs.md)
- [crates/hickory-cli/src/serve/acp/transport.rs](files/crates/hickory-cli/src/serve/acp/transport.rs.md)
- [crates/hickory-cli/src/serve/agent_review.rs](files/crates/hickory-cli/src/serve/agent_review.rs.md)
- [crates/hickory-cli/src/serve/api/actions.rs](files/crates/hickory-cli/src/serve/api/actions.rs.md)
- [crates/hickory-cli/src/serve/api/file_language.rs](files/crates/hickory-cli/src/serve/api/file_language.rs.md)
- [crates/hickory-cli/src/serve/api/snapshot.rs](files/crates/hickory-cli/src/serve/api/snapshot.rs.md)
- [crates/hickory-cli/src/serve/api/tests/tree_tests.rs](files/crates/hickory-cli/src/serve/api/tests/tree_tests.rs.md)
- [crates/hickory-cli/src/serve/bisect.rs](files/crates/hickory-cli/src/serve/bisect.rs.md)
- [crates/hickory-cli/src/serve/bootstrap.rs](files/crates/hickory-cli/src/serve/bootstrap.rs.md)

<book:include file="files/crates/hickory-cli/Cargo.toml.md" />
<book:include file="files/crates/hickory-cli/build.rs.md" />
<book:include file="files/crates/hickory-cli/examples/acp_fixture.rs.md" />
<book:include file="files/crates/hickory-cli/examples/fskit_bundle.rs.md" />
<book:include file="files/crates/hickory-cli/examples/fskit_live.rs.md" />
<book:include file="files/crates/hickory-cli/examples/python_agent_ui_fixture.rs.md" />
<book:include file="files/crates/hickory-cli/src/agent_command.rs.md" />
<book:include file="files/crates/hickory-cli/src/code_model.rs.md" />
<book:include file="files/crates/hickory-cli/src/coverage/commands.rs.md" />
<book:include file="files/crates/hickory-cli/src/coverage/mapping.rs.md" />
<book:include file="files/crates/hickory-cli/src/coverage/mod.rs.md" />
<book:include file="files/crates/hickory-cli/src/coverage/real_tests.rs.md" />
<book:include file="files/crates/hickory-cli/src/coverage/results.rs.md" />
<book:include file="files/crates/hickory-cli/src/coverage/runner.rs.md" />
<book:include file="files/crates/hickory-cli/src/coverage/store.rs.md" />
<book:include file="files/crates/hickory-cli/src/diagnostics.rs.md" />
<book:include file="files/crates/hickory-cli/src/doc_cli_args.rs.md" />
<book:include file="files/crates/hickory-cli/src/engine/client/lifecycle.rs.md" />
<book:include file="files/crates/hickory-cli/src/engine/watch.rs.md" />
<book:include file="files/crates/hickory-cli/src/engine/writes.rs.md" />
<book:include file="files/crates/hickory-cli/src/failed_run.rs.md" />
<book:include file="files/crates/hickory-cli/src/ingest/scratchpad.rs.md" />
<book:include file="files/crates/hickory-cli/src/ingest/tests.rs.md" />
<book:include file="files/crates/hickory-cli/src/init_hook.sh.md" />
<book:include file="files/crates/hickory-cli/src/language_commands.rs.md" />
<book:include file="files/crates/hickory-cli/src/main_thread.rs.md" />
<book:include file="files/crates/hickory-cli/src/mcp/tests/tests.rs.md" />
<book:include file="files/crates/hickory-cli/src/mcp_catalogue.rs.md" />
<book:include file="files/crates/hickory-cli/src/mcp_preview.rs.md" />
<book:include file="files/crates/hickory-cli/src/open_app.rs.md" />
<book:include file="files/crates/hickory-cli/src/output.rs.md" />
<book:include file="files/crates/hickory-cli/src/output_format.rs.md" />
<book:include file="files/crates/hickory-cli/src/performance/browser.rs.md" />
<book:include file="files/crates/hickory-cli/src/performance/mod.rs.md" />
<book:include file="files/crates/hickory-cli/src/performance/native.rs.md" />
<book:include file="files/crates/hickory-cli/src/performance/real_tests.rs.md" />
<book:include file="files/crates/hickory-cli/src/performance/resolve.rs.md" />
<book:include file="files/crates/hickory-cli/src/performance/store.rs.md" />
<book:include file="files/crates/hickory-cli/src/project_command.rs.md" />
<book:include file="files/crates/hickory-cli/src/project_index.rs.md" />
<book:include file="files/crates/hickory-cli/src/python_agent_main.rs.md" />
<book:include file="files/crates/hickory-cli/src/selected_run.rs.md" />
<book:include file="files/crates/hickory-cli/src/serve/acp/transport.rs.md" />
<book:include file="files/crates/hickory-cli/src/serve/agent_review.rs.md" />
<book:include file="files/crates/hickory-cli/src/serve/api/actions.rs.md" />
<book:include file="files/crates/hickory-cli/src/serve/api/file_language.rs.md" />
<book:include file="files/crates/hickory-cli/src/serve/api/snapshot.rs.md" />
<book:include file="files/crates/hickory-cli/src/serve/api/tests/tree_tests.rs.md" />
<book:include file="files/crates/hickory-cli/src/serve/bisect.rs.md" />
<book:include file="files/crates/hickory-cli/src/serve/bootstrap.rs.md" />

</book:doc>
