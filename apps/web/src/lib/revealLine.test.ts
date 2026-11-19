import { afterEach, describe, expect, it, vi } from "vitest";
import { afterSourceTabActivation, claimReveal,claimRevealPosition,onOpenLocation,openCapturedLocation, onRevealLine, resetReveals, revealLine } from "./revealLine";

afterEach(() => {
  resetReveals();
});

describe("asking for a file at a line", () => {
  it("remembers the request for a pane that does not exist yet", () => {
    // A find hit opens a file; the editor for it is built a tick or two
    // later. An event alone would be shouted into an empty room.
    revealLine("src/main.rs", 42);
    expect(claimReveal("src/main.rs")).toBe(42);
  });

  it("is consumed by the first claim, so it does not fire twice", () => {
    // Re-answering on the next re-render would fight the reader for the
    // cursor.
    revealLine("a.txt", 3);
    expect(claimReveal("a.txt")).toBe(3);
    expect(claimReveal("a.txt")).toBeNull();
  });

  it("keeps the newest request for a file, because that is the one clicked", () => {
    revealLine("a.txt", 3);
    revealLine("a.txt", 90);
    expect(claimReveal("a.txt")).toBe(90);
  });

  it("answers nothing for a file nobody asked about", () => {
    expect(claimReveal("never.txt")).toBeNull();
  });

  it("clamps a line number that could not be scrolled to", () => {
    revealLine("a.txt", 0);
    expect(claimReveal("a.txt")).toBe(1);
    revealLine("b.txt", -5);
    expect(claimReveal("b.txt")).toBe(1);
  });

  it("tells listeners which path was asked for", () => {
    const heard = vi.fn();
    const off = onRevealLine(heard);
    revealLine("src/lib.rs", 7);
    expect(heard).toHaveBeenCalledWith("src/lib.rs");
    off();
    revealLine("src/lib.rs", 8);
    expect(heard).toHaveBeenCalledTimes(1);
  });
});
it("retires a queued position when the destination tab loses focus before its editor mounts", () => {
  const origin = {tab: {kind: "file", target: "caller.py"}};
  const target = {kind: "generated", target: "target.py", docId: "owner"};
  let focus = origin;
  const source = "heading\n😀result";
  const request = () => afterSourceTabActivation(target, () => focus, () => true,
    current => revealLine(target.target, 2, 2, {source, current}));
  const first = request();
  expect(first()).toBe(false);
  expect(claimRevealPosition(target.target, source)).toBeNull();
  focus = {tab: target};
  expect(first()).toBe(true);
  focus = origin;
  focus = {tab: target};
  expect(claimRevealPosition(target.target, source)).toBeNull();
  expect(first()).toBe(true);
  expect(claimRevealPosition(target.target, source)).toBeNull();

  expect(request()()).toBe(true);
  expect(claimRevealPosition(target.target, source)).toEqual({line: 2, character: 2});
  expect(claimRevealPosition(target.target, source)).toBeNull();
});

it("retires the handoff when a different generated owner is selected first", () => {
  const target = {kind: "generated", target: "same.py", docId: "owner"};
  let focus = {tab: {kind: "file", target: "caller.py", docId: ""}};
  const deliver = vi.fn();
  const poll = afterSourceTabActivation(target, () => focus, () => true, deliver);
  focus = {tab: {...target, docId: "another-owner"}};
  expect(poll()).toBe(true);
  focus = {tab: target};
  expect(poll()).toBe(true);
  expect(deliver).not.toHaveBeenCalled();
});


it("retains captured source and interaction authority until the final position claim",()=>{
 let current=true;revealLine("a.py",2,0,{source:"a\nb",current:()=>current});current=false;expect(claimRevealPosition("a.py","a\nb")).toBeNull();
 current=true;revealLine("a.py",2,0,{source:"a\nb",current:()=>current});expect(claimRevealPosition("a.py","changed")).toBeNull();
 revealLine("a.py",2,0,{source:"a\nb",current:()=>current});expect(claimReveal("a.py")).toBeNull();
 revealLine("a.py",2,0,{source:"a\nb",current:()=>current});expect(claimRevealPosition("a.py","a\nb")).toEqual({line:2,character:0});expect(claimRevealPosition("a.py","a\nb")).toBeNull();
});
it("never turns an unknown captured mapping identity into a precise source position",()=>{
 const heard=vi.fn(),off=onOpenLocation(heard,true),current=()=>true;openCapturedLocation("helpers.py",99,null,current);expect(heard).toHaveBeenCalledWith(expect.objectContaining({path:"helpers.py",line:99,capture:expect.objectContaining({hash:null,current,positionKnown:true})}));openCapturedLocation("helpers.py",null,"a".repeat(64),current);expect(heard).toHaveBeenLastCalledWith(expect.objectContaining({capture:expect.objectContaining({hash:null,positionKnown:false})}));off();
 const legacy=vi.fn(),retire=onOpenLocation(legacy);openCapturedLocation("helpers.py",99,"a".repeat(64),current,1);expect(legacy).not.toHaveBeenCalled();retire();
});
