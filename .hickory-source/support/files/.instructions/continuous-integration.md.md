<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .instructions/continuous-integration.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `ba091a27aadb937e1dc5541c79c07ac2ffec2c4261b5a28dbf3acfe915f4e82f`.

<book:entry path=".instructions/continuous-integration.md">
# Continuous Integration

- Always impose rules on maximum number of lines in a file or maximum cyclomatic complexity from the very beginning of a project so tech debt accumulates less quickly and to reduce the need for major overhauls right before release.
- Always implement CI and CD from the very beginning of the software project, so they aren't tacked on as an afterthought. Always use Github Actions for CI and CD.
- Always keep pre-commit and pre-push hooks consistent with CI and CD so there is never a case where we're surprised that something passed a pre-commit or pre-push hook but it failed CI or CD in a way that should have been caught by the hook.
</book:entry>

</book:doc>
