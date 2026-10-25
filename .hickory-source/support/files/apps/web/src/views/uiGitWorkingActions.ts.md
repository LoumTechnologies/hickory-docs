<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/uiGitWorkingActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ba715b776d82c5c79e4a4537fce5cb262e6d50ccd9c0e958916829c66648de22`.

<book:entry path="apps/web/src/views/uiGitWorkingActions.ts">
import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import type { GitBranch, GitChanges, GitDiff } from "../api/types";

export function useGitWorkingActions(state: {
  changes: GitChanges | null; busy: string | null; notice: unknown;
  selected: { path: string; staged: boolean } | null; diff: GitDiff | null;
  message: string; amend: boolean; branches: GitBranch[] | null; newBranch: string; confirmDiscard: string | null;
  staged: string[]; unstaged: string[];
  run: (verb: string, action: () => Promise<unknown>, done: string, filesChanged?: boolean) => Promise<UiOutcome>;
  operations: { pull: () => Promise<unknown>; push: () => Promise<unknown>; stash: (pop: boolean) => Promise<unknown>; stage: (paths?: string[]) => Promise<unknown>; unstage: (paths?: string[]) => Promise<unknown>; discard: (path: string) => Promise<unknown>; checkout: (name: string, create?: boolean) => Promise<unknown>; commit: () => Promise<unknown>; review: () => Promise<unknown> };
  openBranches: () => Promise<UiOutcome>;
  setMessage: (message: string) => void; setAmend: (amend: boolean) => void; setNewBranch: (name: string) => void;
  pick: (path: string, staged: boolean) => void; openFile?: (path: string) => void;
  setDiscard: (path: string | null) => void;
}) {
  const parent = `git-working:${useId()}`;
  const revision = useUiRevision(() => [state.changes, state.busy, state.notice, state.selected, state.diff, state.message, state.amend, state.branches, state.newBranch, state.confirmDiscard]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled, reason: enabled ? undefined : "Git busy or control not applicable" }), execute } });
  const complete = (value: Record<string, string | boolean>) => ({ status: "completed" as const, value });
  if (state.changes?.repository) {
    const ready = state.busy === null;
    add("read", "Read Git working tree bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, true, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ changes: state.changes, busy: state.busy, notice: state.notice, selected: state.selected, diff: state.diff, message: state.message, amend: state.amend, branches: state.branches, new_branch: state.branches ? state.newBranch : null, discard_confirmation: state.confirmDiscard, scope: "Last fetched UI Git state; not an atomic disk snapshot or an edit baseline. Command results report actual host replies, not independent verification." }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      return offset > bytes.length ? { status: "unavailable", reason: "Offset outside Git state" } : { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    });
    add("branches", "Toggle Git branches", {}, ready, async () => state.openBranches());
    add("pull", "Pull Git fast-forward", {}, ready, async () => state.run("pull", state.operations.pull, "Pulled", true));
    add("push", "Push Git branch", {}, ready, async (_args, actor) => actor.kind === "agent" ? { status: "requires_input", reason: "Publishing this branch requires a human to use Push" } : state.run("push", state.operations.push, "Pushed"));
    add("stash", "Stash Git working changes", {}, ready && state.changes.files.length > 0, async () => state.run("stash", () => state.operations.stash(false), "Stashed", true));
    add("pop", "Pop Git stash", {}, ready, async () => state.run("stash", () => state.operations.stash(true), "Stash popped", true));
    for (const [staged, paths] of [[true, state.staged], [false, state.unstaged]] as const) {
      const verb = staged ? "unstage" : "stage", title = staged ? "Unstage" : "Stage";
      const effect = staged ? state.operations.unstage : state.operations.stage;
      if (paths.length) add(`${verb}:all`, `${title} all Git files`, {}, ready, async () => state.run(verb, () => effect(), `${title}d everything`));
      for (const [index, path] of paths.entries()) {
        const key = `${staged ? "staged" : "unstaged"}:${index}`;
        add(`pick:${key}`, `Show ${staged ? "staged" : "unstaged"} diff ${path}`, {}, true, async () => { state.pick(path, staged); return complete({ diff_requested: true }); });
        if (state.openFile) add(`open:${key}`, `Open current Git file ${path}`, {}, true, async () => { state.openFile!(path); return complete({ navigation_requested: true }); });
        add(`${verb}:${index}`, `${title} Git file ${path}`, {}, ready, async () => state.run(verb, () => effect([path]), `${title}d ${path}`));
        if (!staged) add(`discard:${index}`, `Discard Git changes ${path}`, {}, ready, async (_args, actor) => {
          if (actor.kind === "agent") return { status: "requires_input", reason: "Discard requires the human's two-click confirmation on this file" };
          if (state.confirmDiscard !== path) { state.setDiscard(path); return complete({ confirmation_requested: true, discarded: false }); }
          state.setDiscard(null); return state.run("discard", () => state.operations.discard(path), `Discarded the changes to ${path}`, true);
        });
      }
    }
    add("message", "Set Git commit message", { text: { type: "string", maxLength: 1048576 } }, ready, async args => { state.setMessage(args.text as string); return complete({ draft_changed: true }); });
    add("amend", "Set Git amend mode", { enabled: { type: "boolean" } }, ready, async args => { state.setAmend(args.enabled as boolean); return complete({ draft_changed: true }); });
    add("commit", state.amend ? "Amend Git commit" : "Create Git commit", {}, ready && !!state.message.trim() && (state.amend || state.staged.length > 0), async (_args, actor) => state.amend && actor.kind === "agent" ? { status: "requires_input", reason: "Rewriting the last commit requires the human to use Amend" } : state.run("commit", state.operations.commit, state.amend ? "Amended the last commit" : "Committed"));
    if (state.branches) {
      add("branch-name", "Set new Git branch name", { name: { type: "string", maxLength: 4096 } }, ready, async args => { state.setNewBranch(args.name as string); return complete({ draft_changed: true }); });
      add("branch-create", "Create and switch Git branch", {}, ready && !!state.newBranch.trim(), async () => state.run("checkout", () => state.operations.checkout(state.newBranch.trim(), true), `On ${state.newBranch.trim()}`, true));
      for (const [index, branch] of state.branches.entries()) add(`checkout:${index}`, `Switch Git branch ${branch.name}`, {}, ready && !branch.current, async () => state.run("checkout", () => state.operations.checkout(branch.name), `On ${branch.name}`, true));
    }
    if (state.selected && state.diff && state.diff.path === state.selected.path) add("review", "Review Git diff in main editor", {}, ready, async () => state.run("review", state.operations.review, "Opened editable review"));
  }
  const invoke = useUiCommands(commands);
  return (key: string, args: Parameters<typeof invoke>[1] = {}) => invoke(`${parent}:${key}`, args).catch(() => undefined);
}
</book:entry>

</book:doc>
