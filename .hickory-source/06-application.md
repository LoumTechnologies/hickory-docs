<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Assemble the application and select its checks

For engineers connecting the core engine to its user-facing hosts.

The same language and project model support the command line, browser and desktop.
Finish construction at those boundaries, then select focused checks and distribution
policy. The rest of each host remains in the component reference.

## Start the application through hick

The binary parses the product command surface and dispatches into shared implementations. Repo-specific dogfood checks remain outside this command enum. Read a command's argument type beside its dispatch before changing defaults or execution behavior.

<book:include file="mechanisms/crates/hickory-cli/src/main.rs.md" />

## Mount the shared browser application

The browser host composes the document workspace and its application state. Document editing and generated-output editing use the shared components from the editing chapter rather than separate language implementations.

<book:include file="mechanisms/apps/web/src/App.tsx.md" />

## Host the same application on desktop

The Tauri boundary owns native windows, OS events and the local engine lifecycle. It supplies host behavior around the shared web application, rather than a second editor or construction engine.

<book:include file="mechanisms/apps/desktop/src-tauri/src/lib.rs.md" />

## Select evidence beside the behavior

The CI policy constructs its actual JSON input below. Check definitions remain
beside the behavior they validate; selecting an ID does not establish a passing
run. Follow the [release contracts](guarantees/release/index.md) when publishing
a build. Broad regression execution and post-commit monitoring remain separate
from this narrative-only refactor.

<book:include file="ci.md" />

Continue with the [component reference](mechanisms/index.md) for additional
execution backends, language services, agents and integrations. The
[construction entry](../project.md) provides the primary reading order.
</book:doc>
