<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/BlankWelcome.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e391afabc0d60e989ed9b7336d46f3c79a165ce41572a6f221041d8320e219e8`.

<book:entry path="apps/web/src/views/BlankWelcome.tsx">
// File → New Window opens this intentionally small surface. It has no API
// behind it: until a person chooses a file or folder, there is no workspace to
// list, modify, lock, or accidentally remember.
export function BlankWelcome() {
  return (
    <section className="welcome" aria-label="Welcome">
      <div className="welcome__inner">
        <header className="welcome__head">
          <h1 className="welcome__title">Hickory Docs</h1>
          <p className="welcome__sub">A new window, with nothing open.</p>
        </header>
        <div className="welcome__columns">
          <section className="welcome__col" aria-label="Start">
            <h2 className="welcome__h2">Start</h2>
            <p className="muted welcome__empty">
              Choose <strong>File → Open Folder…</strong> to open a notes folder, or{" "}
              <strong>File → Open File…</strong> to open one document.
            </p>
          </section>
        </div>
      </div>
    </section>
  );
}
</book:entry>

</book:doc>
