<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/BlankWelcome.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-302f315b6ad0"></a>
## `apps/web/src/views/BlankWelcome.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-302f315b6ad0-BlankWelcome"></a>
### `BlankWelcome`

File → New Window opens this intentionally small surface. It has no API
behind it: until a person chooses a file or folder, there is no workspace to
list, modify, lock, or accidentally remember.

<storycode:copy id="f-302f315b6ad0-BlankWelcome">
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
</storycode:copy>

### Assemble `apps/web/src/views/BlankWelcome.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/BlankWelcome.tsx"><storycode:paste select="#f-302f315b6ad0-BlankWelcome"/></storycode:file>

</storycode:doc>
