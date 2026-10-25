<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .instructions/user-facing-errors.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `c3f72208e49c17e8d7a1d04cdea638ccf3b21bf058c47501f05438edec3e5393`.

<book:entry path=".instructions/user-facing-errors.md">
# User-facing error messages

- Error messages for **user-facing** surfaces must be clear and easy to understand.
- Always offer **possible next steps** the user can take.
- When useful, mention **common misconfigurations or mistakes** that often cause this error.
- Include **relevant context** that helps the user fix the problem (paths, IDs, which check failed, which value was rejected, which dependency is missing).
- Surfaces that count as user-facing: command-line programs, graphical UIs, installers, and HTTP APIs or status pages meant for human operators.
- The only reason to omit extra diagnostic context is when gathering it is **performance-expensive** and the code is a **library** used in hot paths — not a user-facing tool. In that case prefer a clear message plus a documented way to get more detail (flag, log level, or error code).
</book:entry>

</book:doc>
