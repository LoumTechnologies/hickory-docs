import { createRoot } from "react-dom/client";
import homeSource from "../../../../.hickory-source/guarantees/landing/read-only-homepage/home.md?raw";
import { Buffer, Buffers } from "./buffers";
import { Document } from "./Document";
import { RichOwners } from "./RichInline";
import "./simple.css";

function browserLink(href: string) {
  try {
    const url = new URL(href, window.location.href);
    return url.protocol === "https:" || (url.protocol === "http:" && url.origin === window.location.origin)
      ? url.href : null;
  } catch {
    return null;
  }
}

const buffers = new Buffers(() => {});
const file = { name: "home.md", path: "home.md", dir: false, generated_by: "published site" };
const buffer = new Buffer(file, homeSource, "", buffers.changed, true);
buffers.values.set(file.path, buffer);

createRoot(document.getElementById("root")!).render(
  <main className="site-home">
    <aside className="group" aria-label="Files">
      <div className="tabs"><span>Files</span></div>
      <div className="file-tree"><ul><li aria-current="true">{file.name}</li></ul></div>
    </aside>
    <section className="group" aria-label={file.name}>
      <div className="tabs"><div className="tab" data-active="true"><span>{file.name}</span></div></div>
      <div className="pane-with-status">
        <Document buffer={buffer} />
        <footer className="pane-status" aria-label="Document status">Read-only</footer>
      </div>
    </section>
    <RichOwners buffers={buffers} browserLink={browserLink} />
  </main>,
);
