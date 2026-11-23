<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/developers/developer-environment.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `8ae8b99f34fbd1609eb0124c942c1fda60d76f9dc43b42529b2989ea8acf90a7`.

<book:entry path="docs/developers/developer-environment.md">
# Develop Hickory locally

Run `just dev` from this checkout. It opens the Simple native editor on macOS
and the Simple browser elsewhere. Both use an isolated seed workspace under
`.dev/project`. Set `HICKORY_PROJECT_DIR` to use an existing folder and
`HICKORY_OPEN_FILE` to open a specific workspace-relative file.

Use `just dev browser`, `just dev apple`, `just dev windows` or
`just dev linux` to choose a desktop frontend explicitly. The supervisor
rebuilds and relaunches that frontend when HEAD changes. Use `just dev-stop`
to stop it, or `just dev-clean` to also remove this checkout's development
scratch directory. Save or move any scratch edits you intend to keep first.

Run `just dev-site` to work on the public site and its embedded demonstrations.
The site has separate development and build entry points.

The [Simple frontend guide](simple-frontends.md) lists prerequisites for each
platform, including the iOS simulator and Android. On macOS, use
`just local-install` to install a signed local build in Applications.
</book:entry>

</book:doc>
