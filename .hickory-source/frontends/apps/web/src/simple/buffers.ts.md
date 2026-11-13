<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep buffer revisions independent of pane arrangement

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Each buffer owns its text and save baseline. Subscribers to text changes are separate from the workspace shell so typing does not rerender every pane. One in-flight read per path prevents late duplicate opens from replacing edits.

Managed documents use their document ID to read the current source directly, without rendering or analysis. The plain-file endpoint deliberately refuses them. Their save baseline is that source text; the hash is unused for document saves.

<ui:copy id="simple-964718fb75-0">
import { api } from "../api/client";
import type { FileNode } from "../api/types";
export class Buffer {
  text: string;
  base: string;
  hash: string;
  saving = false;
  error = "";
  version = 0;
  private listeners = new Set<() => void>();
  constructor(readonly file: FileNode, content: string, hash: string, private shellChanged: () => void, readonly transient = false) {
    this.text = this.base = content; this.hash = hash;
  }
  get dirty() { return this.text !== this.base; }
  get readonly() { return !!this.file.generated_by; }
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  snapshot = () => this.version;
  publish() { this.version++; this.listeners.forEach(listener => listener()); }
  edit(text: string) {
    if (this.readonly) return;
    const dirty = this.dirty; this.text = text; this.publish();
    if (dirty !== this.dirty) this.shellChanged();
  }
</ui:copy>

Capture text and revision once when Save is pressed. Never fetch a new baseline before writing, force an overwrite, or mark later keystrokes as saved. Literate writes use the existing room-and-disk check.

<ui:copy id="simple-964718fb75-1">
  async save() {
    if (this.transient || this.readonly || this.saving || !this.dirty) return;
    const text = this.text, base = this.base, hash = this.hash;
    this.saving = true; this.error = ""; this.shellChanged(); this.publish();
    try {
      if (this.file.doc_id) await api.saveDoc(this.file.doc_id, text, { source: base, disk: base });
      else this.hash = (await api.saveFile(this.file.path, text, hash)).hash;
      this.base = text;
    } catch (error) {
      this.error = `${String(error)} Your edits are retained. Copy them before closing; compare with the file on disk before retrying.`;
    } finally { this.saving = false; this.publish(); this.shellChanged(); }
  }
}
export class Buffers {
  values = new Map<string, Buffer>();
  private pending = new Map<string, Promise<Buffer>>();
  constructor(private changed: () => void) {}
  open(file: FileNode): Promise<Buffer> {
    const current = this.values.get(file.path);
    if (current) return Promise.resolve(current);
    const pending = this.pending.get(file.path);
    if (pending) return pending;
    const read = file.doc_id
      ? api.doc(file.doc_id).then(doc => ({ content: doc.source, hash: "" }))
      : api.file(file.path);
    const request = read.then(data => {
      const buffer = new Buffer(file, data.content, data.hash, this.changed);
      this.values.set(file.path, buffer); return buffer;
    }).finally(() => this.pending.delete(file.path));
    this.pending.set(file.path, request); return request;
  }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/buffers.ts"><ui:paste select="#simple-964718fb75-0"/><ui:paste select="#simple-964718fb75-1"/></ui:file>

</ui:doc>
