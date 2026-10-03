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
