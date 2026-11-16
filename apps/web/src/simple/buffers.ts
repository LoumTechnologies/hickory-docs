import { api } from "../api/client";
import type { FileNode } from "../api/types";
export class Buffer {
  text: string;
  base: string;
  hash: string;
  saving = false;
  error = "";
  completionNotice="";
  setCompletionNotice(value:string){if(this.completionNotice===value)return;this.completionNotice=value;this.publish();this.shellChanged();}
  version = 0;
  sourceRevision=0;
  private listeners = new Set<() => void>();
  resourceDeleted = false;
  constructor(public file: FileNode, content: string, hash: string, private shellChanged: () => void, readonly transient = false,public namespace = "") {
    this.text = this.base = content; this.hash = hash;
  }
  get dirty() { return this.text !== this.base; }
  get readonly() { return !!this.file.generated_by; }
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  snapshot = () => this.version;
  publish() { this.version++; this.listeners.forEach(listener => listener()); }
  edit(text: string) {
    if (this.readonly) return;
    const dirty = this.dirty;if(this.text!==text)this.sourceRevision++; this.text = text; this.publish();
    if (dirty !== this.dirty) this.shellChanged();
  }
  async save() {
    if (this.resourceDeleted || this.transient || this.readonly || this.saving || !this.dirty) return;
    const text = this.text, base = this.base, hash = this.hash;
    this.saving = true; this.error = ""; this.shellChanged(); this.publish();
    try {
      if (this.file.doc_id) await api.saveDoc(this.file.doc_id, text, { source: base, disk: base });
      else this.hash = (await api.saveFile(this.file.path, text, hash)).hash;
      this.base = text;this.sourceRevision++;
    } catch (error) {
      this.error = `${String(error)} Your edits are retained. Copy them before closing; compare with the file on disk before retrying.`;
    } finally { this.saving = false; this.publish(); this.shellChanged(); }
  }
}
export class Buffers {
  private namespace="";
  get workspace(){return this.namespace;}
  set workspace(value:string){const initial=!this.namespace;this.namespace=value;if(initial&&value)for(const buffer of this.values.values())if(!buffer.namespace){buffer.namespace=value;buffer.sourceRevision++;buffer.publish();}}

  values = new Map<string, Buffer>();
  retainedResources = new Map<string, Buffer>();
  private namespaceRevision = 0;
  retirePendingReads() { this.namespaceRevision++; this.pending.clear(); }
  get hasUnsavedChanges() { return [...this.values.values(),...this.retainedResources.values()].some(buffer=>buffer.dirty||buffer.saving); }
  private pending = new Map<string, Promise<Buffer>>();
  constructor(public changed: () => void) {}
  open(file: FileNode): Promise<Buffer> {
    const current = this.values.get(file.path);
    if (current) return Promise.resolve(current);
    const pending = this.pending.get(file.path);
    if (pending) return pending;
    const revision = this.namespaceRevision;
    const read = file.doc_id
      ? api.doc(file.doc_id).then(doc => ({ content: doc.source, hash: "" }))
      : api.file(file.path);
    const request = read.then(data => {
      if (revision !== this.namespaceRevision) throw new Error("Source addresses changed while opening. Choose the file again.");
      const available = this.values.get(file.path); if (available) return available;
      const buffer = new Buffer(file, data.content, data.hash, this.changed,false,this.workspace);
      this.values.set(file.path, buffer); this.changed(); return buffer;
    }).finally(() => { if(this.pending.get(file.path)===request)this.pending.delete(file.path); });
    this.pending.set(file.path, request); return request;
  }
}
