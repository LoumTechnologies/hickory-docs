<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/fileMenuActions.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Filesystem Acts Keep Dired's Safety](../../../../../contract.md)

Captured SHA-256: `6cfe0a9d23b6fae98d091d665eaa289f8651756ee66ee18ca854f672cec6613e`.

<book:entry path="apps/web/src/shell/fileMenuActions.tsx">
// File actions and their prompts are shared by Files rows and file tabs.
import { api } from "../api/client";
import { FILES_CHANGED_EVENT } from "./FolderTreePane";
import { TreePrompt } from "./TreePrompt";
import { fileRenameError, TreeRenameEditor } from "./TreeRenameEditor";
import type { Dired } from "./useDired";
import { copyText, type TreeMenuItem } from "./treeMenu";

export function runFileMenuItem(item: TreeMenuItem, dired: Dired, notice: (text: string) => void, openDocument: (id: string) => void): boolean {
  const action = item.action;
  const failed = (e: unknown) => notice(e instanceof Error ? e.message : String(e));
  switch (action.kind) {
    case "copy": void copyText(action.text).catch(failed); return true;
    case "reveal": void api.reveal(action.path).catch(failed); return true;
    case "open": void api.openExternal(action.path).catch(failed); return true;
    case "literate":
    case "ingest":
      void api.adopt(action.path, action.kind === "ingest" ? action.into : undefined).then(outcome => {
        window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
        openDocument(outcome.doc_id);
      }, failed);
      return true;
    case "rename":
    case "move":
    case "delete": dired.run({ kind: action.kind, paths: action.paths }); return true;
    case "copy-to": dired.run({ kind: "copy", paths: action.paths }); return true;
    case "create":
    case "mkdir": dired.run({ kind: action.kind, dir: action.dir }); return true;
    default: return false;
  }
}

export function FileActionPrompts({ dired, inlineRename = false }: { dired: Dired; inlineRename?: boolean }) {
  return <>
      {dired.prompt && (
        <TreePrompt
          owner={dired.prompt.id} subject={dired.uiSubject} value={dired.promptDraft} onValueChange={dired.setPromptDraft}
          label={dired.prompt.label}
          verb={dired.prompt.verb}
          initial={dired.prompt.initial}
          onSubmit={dired.answer}
          onCancel={dired.cancel}
          error={dired.promptError}
          busy={dired.busy}
        />
      )}
      {dired.renamePaths && (
        <TreeRenameEditor
          items={dired.renamePaths.map((path) => ({
            key: `filesystem:${encodeURIComponent(path)}`,
            context: path,
            value: path.split("/").pop() ?? path,
          }))}
          validate={fileRenameError}
          inlinePath={inlineRename && dired.renamePaths.length === 1 ? dired.renamePaths[0] : undefined}
          apply={async (item, value) => {
            await api.fileOp({ op: "rename", path: item.context, to: value });
            const slash = item.context.lastIndexOf("/");
            const to = slash < 0 ? value : `${item.context.slice(0, slash + 1)}${value}`;
            window.dispatchEvent(
              new CustomEvent(FILES_CHANGED_EVENT, { detail: { renames: [{ from: item.context, to }] } }),
            );
          }}
          onClose={dired.closeRename}
        />
      )}
  </>;
}
</book:entry>

</book:doc>
