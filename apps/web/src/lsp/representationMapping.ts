import type { Representation } from "../api/representations";
import { byteToChar, charToByte } from "../lib/offsets";
import { positionToUtf16, utf16ToPosition, type LspPosition } from "./positions";

export type RepresentedFile = Representation["files"][number];
// Only exact literal/paste mappings are writable. Never guess across a gap.
export function filePosition(source: string, file: RepresentedFile, offset: number): LspPosition | null {
  const at = charToByte(source, offset);
  const matches = file.provenance.filter(p => ["literal", "paste", "ingested"].includes(p.origin.kind) && "span" in p.origin && p.origin.span && (!file.source_path || ("doc_path" in p.origin && p.origin.doc_path === file.source_path)) && p.end - p.start === p.origin.span[1] - p.origin.span[0] && at >= p.origin.span[0] && at < p.origin.span[1]);
  if (!matches.length) matches.push(...file.provenance.filter(p => ["literal", "paste", "ingested"].includes(p.origin.kind) && "span" in p.origin && p.origin.span && (!file.source_path || ("doc_path" in p.origin && p.origin.doc_path === file.source_path)) && at === p.origin.span[1] && p.end - p.start === p.origin.span[1] - p.origin.span[0]));
  if (matches.length !== 1) return null;
  const p = matches[0];
  if (!("span" in p.origin) || !p.origin.span) return null;
  const delta = at - p.origin.span[0];
  if (delta > p.end - p.start) return null;
  return utf16ToPosition(file.content, byteToChar(file.content, p.start + delta));
}
export function representationOffset(source: string, file: RepresentedFile, position: LspPosition): number | null {
  const at = charToByte(file.content, positionToUtf16(file.content, position));
  const matches = file.provenance.filter(p => at >= p.start && at < p.end && ["literal", "paste", "ingested"].includes(p.origin.kind) && "span" in p.origin && p.origin.span && (!file.source_path || ("doc_path" in p.origin && p.origin.doc_path === file.source_path)) && p.end - p.start === p.origin.span[1] - p.origin.span[0]);
  if (!matches.length && at === charToByte(file.content, file.content.length)) matches.push(...file.provenance.filter(p => p.end === at && ["literal", "paste", "ingested"].includes(p.origin.kind) && "span" in p.origin && p.origin.span && (!file.source_path || ("doc_path" in p.origin && p.origin.doc_path === file.source_path)) && p.end - p.start === p.origin.span[1] - p.origin.span[0]));
  if (matches.length !== 1) return null;
  const p = matches[0];
  if (!("span" in p.origin) || !p.origin.span) return null;
  const delta = at - p.start;
  if (delta > p.origin.span[1] - p.origin.span[0]) return null;
  return byteToChar(source, p.origin.span[0] + delta);
}

/** An edit can span fragments only when their source bytes are contiguous. */
export function representationRange(source:string, file:RepresentedFile, range:{start:LspPosition;end:LspPosition}): {from:number;to:number}|null {
  const start=charToByte(file.content,positionToUtf16(file.content,range.start));
  const end=charToByte(file.content,positionToUtf16(file.content,range.end));
  if (end<start) return null;
  if (start===end) { const from=representationOffset(source,file,range.start); return from===null?null:{from,to:from}; }
  let at=start, first:number|null=null, previous:number|null=null;
  for (const p of file.provenance) {
    if(p.end<=at || p.start>=end) continue;
    if(p.start>at || !["literal","paste","ingested"].includes(p.origin.kind) || !("span" in p.origin) || !p.origin.span || p.end-p.start!==p.origin.span[1]-p.origin.span[0] || (file.source_path && (!("doc_path" in p.origin)||p.origin.doc_path!==file.source_path))) return null;
    const from=p.origin.span[0]+at-p.start;
    if(previous!==null && previous!==from) return null;
    if(first===null) first=from;
    const until=Math.min(end,p.end); previous=from+until-at; at=until;
    if(at===end) break;
  }
  if(at!==end || first===null || previous===null) return null;
  const from=byteToChar(source,first),to=byteToChar(source,previous);
  return source.slice(from,to)===file.content.slice(byteToChar(file.content,start),byteToChar(file.content,end))?{from,to}:null;
}
