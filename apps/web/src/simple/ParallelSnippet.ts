import {
  advanceSnippet, beginSnippet, editSnippet, nestSnippet, snippetMetadataCost,
  type SnippetSession,
} from './SnippetSession';
import type {SnippetField} from './SnippetExpansion';

export interface SnippetRange {from: number; to: number}
export interface ParallelReplacement extends SnippetRange {text: string}
export interface ParallelExpansion extends ParallelReplacement {fields: readonly SnippetField[]}

interface Peer extends SnippetRange {
  id: number;
  text: string;
  selection: SnippetRange;
  session: SnippetSession | null;
}

export interface ParallelSnippet {
  source: string;
  peers: readonly Peer[];
  primary: number;
}

export interface ParallelSnippetChange {
  edits: readonly ParallelReplacement[];
  state: ParallelSnippet;
}

const maxCarets = 256;
const maxFields = 4096;
const maxBytes = 1024 * 1024;
const encoder = new TextEncoder();

function scalarText(text: string): boolean {
  for (let at = 0; at < text.length; at++) {
    const unit = text.charCodeAt(at);
    if (unit >= 0xd800 && unit <= 0xdbff) {
      const next = text.charCodeAt(++at);
      if (!(next >= 0xdc00 && next <= 0xdfff)) return false;
    } else if (unit >= 0xdc00 && unit <= 0xdfff) {
      return false;
    }
  }
  return true;
}

function boundary(text: string, at: number): boolean {
  if (!Number.isSafeInteger(at) || at < 0 || at > text.length) return false;
  if (at === 0 || at === text.length) return true;
  const left = text.charCodeAt(at - 1);
  const right = text.charCodeAt(at);
  return !(left === 13 && right === 10 ||
    left >= 0xd800 && left <= 0xdbff && right >= 0xdc00 && right <= 0xdfff);
}

function valid(text: string, range: SnippetRange): boolean {
  return range.from <= range.to && boundary(text, range.from) && boundary(text, range.to);
}

function sameRange(a: SnippetRange, b: SnippetRange): boolean {
  return a.from === b.from && a.to === b.to;
}

function replace(source: string, edits: readonly ParallelReplacement[]): string {
  const parts: string[] = [];
  let start = 0;
  for (const edit of edits) {
    parts.push(source.slice(start, edit.from), edit.text);
    start = edit.to;
  }
  parts.push(source.slice(start));
  return parts.join('');
}

function current(state: ParallelSnippet, source: string): boolean {
  return state.source === source && state.peers.every((peer, index) =>
    valid(source, peer) && source.slice(peer.from, peer.to) === peer.text &&
    valid(peer.text, peer.selection) && (!peer.session || peer.session.source === peer.text) &&
    (index === 0 || state.peers[index - 1].to <= peer.from)
  );
}

export function parallelSelections(state: ParallelSnippet): {
  ranges: SnippetRange[];
  primary: number;
} {
  return {
    ranges: state.peers.map(peer => ({
      from: peer.from + peer.selection.from, to: peer.from + peer.selection.to,
    })),
    primary: state.peers.findIndex(peer => peer.id === state.primary),
  };
}

// The completion planner has already combined and checked the shared edits.
// Map the peer envelopes into that final draft and verify their literal text.
export function rebaseParallelSnippet(
  state: ParallelSnippet, source: string, starts: readonly number[],
): ParallelSnippet | null {
  if (starts.length !== state.peers.length) return null;
  const peers = state.peers.map(peer => ({
    ...peer, from: starts[peer.id], to: starts[peer.id] + peer.text.length,
  }));
  const rebased = {...state, source, peers};
  const {ranges} = parallelSelections(rebased);
  if (ranges.some((range, index) => index > 0 &&
      (ranges[index - 1].to > range.from || sameRange(ranges[index - 1], range)))) return null;
  return current(rebased, source) ? rebased : null;
}

export function beginParallelSnippet(
  source: string, expansions: readonly ParallelExpansion[], primary: number,
): ParallelSnippetChange | null {
  if (source.length > 8 * maxBytes || expansions.length < 2 || expansions.length > maxCarets ||
      !Number.isInteger(primary) || primary < 0 || primary >= expansions.length ||
      !scalarText(source) || encoder.encode(source).length > 8 * maxBytes) return null;

  const ordered = expansions.map((edit, id) => ({edit, id}))
    .sort((a, b) => a.edit.from - b.edit.from || a.edit.to - b.edit.to);
  const peers: Peer[] = [];
  const edits: ParallelReplacement[] = [];
  const budget = {work: 0};
  let bytes = 0;
  let metadata = 0;
  let fields = 0;
  let shift = 0;
  for (const {edit, id} of ordered) {
    const previous = edits.at(-1);
    if (!valid(source, edit) || previous && (previous.to > edit.from || sameRange(previous, edit)) ||
        edit.text.length > maxBytes - bytes || !scalarText(edit.text)) return null;
    bytes += encoder.encode(edit.text).length;
    fields += edit.fields.length;
    const cost = snippetMetadataCost(edit.fields);
    if (cost === null || bytes > maxBytes || fields > maxFields || cost > maxBytes - metadata) return null;
    metadata += cost;

    const session = beginSnippet(edit.text, edit.fields, budget);
    const exit = edit.fields.find(field => field.index === 0);
    if (!exit || !valid(edit.text, exit) || exit.from !== exit.to) return null;
    // An expansion without editable fields still keeps its own terminal caret.
    if (!session && (edit.fields.length !== 1 || exit.transform || exit.parent != null ||
        exit.id == null || !Number.isSafeInteger(exit.id) || exit.id < 0 || exit.id >= maxFields)) return null;
    const field = session?.fields.find(field => field.id === session.current) ?? exit;
    const from = edit.from + shift;
    peers.push({
      id, from, to: from + edit.text.length, text: edit.text, session,
      selection: {from: field.from, to: field.to},
    });
    edits.push({from: edit.from, to: edit.to, text: edit.text});
    shift += edit.text.length - (edit.to - edit.from);
  }

  const after = replace(source, edits);
  const state = {source: after, peers, primary};
  return current(state, after) ? {edits, state} : null;
}

function selectPeer(peer: Peer, absolute: SnippetRange): Peer | null {
  const selection = {from: absolute.from - peer.from, to: absolute.to - peer.from};
  if (!valid(peer.text, selection)) return null;
  if (!peer.session || peer.session.active === 0) return {...peer, selection};

  const session = peer.session;
  const match = session.fields.find(field => field.id === session.current &&
    field.from <= selection.from && selection.to <= field.to);
  if (!match || session.fields.some(field => field.transform && field.from < field.to &&
      field.from <= selection.from && selection.to <= field.to &&
      (selection.from < selection.to || field.from < selection.from && selection.from < field.to))) return null;
  return {...peer, selection};
}

export function selectParallelSnippet(
  state: ParallelSnippet, source: string, ranges: readonly SnippetRange[], primary: number,
): ParallelSnippet | null {
  if (!current(state, source) || ranges.length !== state.peers.length) return null;
  const main = state.peers[primary];
  if (!main) return null;
  const peers: Peer[] = [];
  for (const [index, peer] of state.peers.entries()) {
    const selected = selectPeer(peer, ranges[index]);
    if (!selected) return null;
    peers.push(selected);
  }
  return {...state, peers, primary: main.id};
}

// Ranges are in original source coordinates in peer order. An editor can supply
// different deletions or pasted fragments. Construct every peer before applying.
export function editParallelSnippet(
  state: ParallelSnippet, source: string, input: readonly ParallelReplacement[],
): ParallelSnippetChange | null {
  return prepareParallelInput(state, source, input);
}

export function planParallelSnippet(
  source: string, expansions: readonly ParallelExpansion[], primary: number,
  continuing: ParallelSnippet | null = null,
): ParallelSnippetChange | null {
  if (!continuing) return beginParallelSnippet(source, expansions, primary);
  const previous = parallelSelections(continuing);
  if (primary !== previous.primary || expansions.length !== previous.ranges.length ||
      expansions.some((row, index) => !sameRange(row, previous.ranges[index]))) return null;
  return prepareParallelInput(continuing, source, expansions);
}

function prepareParallelInput(
  state: ParallelSnippet, source: string,
  input: readonly (ParallelExpansion | ParallelReplacement)[],
): ParallelSnippetChange | null {
  if (!current(state, source) || input.length !== state.peers.length) return null;

  const budget = {work: 0};
  const peers: Peer[] = [];
  const edits: ParallelReplacement[] = [];
  let shift = 0;
  let bytes = 0;
  let metadata = 0;
  let fields = 0;
  for (const [index, peer] of state.peers.entries()) {
    const replacement = input[index];
    const selected = selectPeer(peer, replacement);
    if (!selected || replacement.text.length > maxBytes - bytes ||
        !scalarText(replacement.text)) return null;
    const {from, to} = selected.selection;
    const local = {from, to, text: replacement.text};
    const expansion = 'fields' in replacement ? replacement : null;
    if (expansion) {
      const exit = expansion.fields.find(field => field.index === 0);
      if (!exit || exit.from !== exit.to || exit.id == null || !Number.isSafeInteger(exit.id) ||
          exit.id < 0 || exit.id >= maxFields || exit.transform || expansion.fields.length > maxFields ||
          expansion.fields.some(field => !valid(expansion.text, field))) return null;
    }
    let plan;
    if (peer.session?.active) {
      plan = expansion ?
        nestSnippet(peer.session, peer.text, local, [], expansion.fields, budget) :
        editSnippet(peer.session, peer.text, from, to, replacement.text, budget);
      if (!plan) return null;
    }
    const text = plan?.session.source ?? replace(peer.text, [local]);
    const caret = from + replacement.text.length;
    let selection = plan?.selection ?? {from: caret, to: caret};
    let session = plan?.session ?? null;
    if (!plan && expansion) {
      const exit = expansion.fields.find(field => field.index === 0);
      if (!exit || !valid(expansion.text, exit) || exit.from !== exit.to) return null;
      const shifted = expansion.fields.map(field => ({
        ...field, from: field.from + from, to: field.to + from,
      }));
      session = beginSnippet(text, shifted, budget);
      if (!session && (shifted.length !== 1 || exit.transform || exit.parent != null ||
          exit.id == null || !Number.isSafeInteger(exit.id) || exit.id < 0 || exit.id >= maxFields)) return null;
      const first = session?.fields.find(field => field.id === session!.current);
      selection = first ? {from: first.from, to: first.to} : {from: from + exit.from, to: from + exit.to};
    }
    if (expansion) {
      const retained = session?.fields ?? [];
      const cost = snippetMetadataCost(retained);
      fields += retained.length;
      if (cost === null || cost > maxBytes - metadata || fields > maxFields) return null;
      metadata += cost;
    }
    const start = peer.from + shift;
    bytes += encoder.encode(text).length;
    if (bytes > maxBytes) return null;
    peers.push({id: peer.id, from: start, to: start + text.length, text, selection, session});
    shift += text.length - peer.text.length;

    for (const replacement of plan?.edits ?? [local]) {
      const edit = {from: peer.from + replacement.from, to: peer.from + replacement.to, text: replacement.text};
      const previous = edits.at(-1);
      if (previous && previous.to > edit.from) return null;
      if (previous && previous.to === edit.from) {
        // Adjacent and coincident empty contributions retain original peer order.
        previous.to = edit.to;
        previous.text += edit.text;
      } else {
        edits.push(edit);
      }
    }
  }

  const after = replace(source, edits);
  const updated = {...state, source: after, peers};
  return current(updated, after) ? {edits, state: updated} : null;
}

export function moveParallelSnippet(
  state: ParallelSnippet, source: string, direction: 1 | -1,
): ParallelSnippet | null {
  if (!current(state, source)) return null;
  const peers: Peer[] = [];
  for (const peer of state.peers) {
    const {session, text, selection} = peer;
    if (!session) {
      peers.push(peer);
      continue;
    }
    const next = advanceSnippet(session, text, selection.from, selection.to, direction);
    if (!next) {
      const first = session.scopes?.at(-1)?.order[0] ??
        Math.min(...session.fields.filter(field => !field.transform && field.index > 0).map(field => field.index));
      if (!(direction === 1 && session.active === 0 || direction === -1 && session.active === first)) return null;
      peers.push(peer);
      continue;
    }
    const exit = session.fields.find(field => field.index === 0)!;
    peers.push({
      ...peer, selection: next.selection,
      session: next.session ?? {...session, active: 0, current: exit.id!},
    });
  }
  return {...state, peers};
}
