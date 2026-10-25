<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/OutputTree.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e5d6e9e65e0d3a78f6790e3c8980d24aa842e0ec6099309065e343db537d72ae`.

<book:entry path="apps/web/src/components/OutputTree.tsx">
// The middle column of the Split view: the tree of files this document
// generates. It is a waypoint, not a sidebar — lineage ribbons enter each file
// node from the document on the left and leave it into the generated text on
// the right, so the picture reads "this prose becomes these bytes of that
// file". Rows register their DOM node so SplitView can measure the band a
// ribbon should pass through.

import { useMemo } from "react";
import type { OutputFileMeta } from "../api/types";
import { buildTree, type DirNode } from "../lib/tree";

export interface OutputTreeProps {
  files: OutputFileMeta[];
  activePath: string | null;
  onSelect: (path: string) => void;
  /** Per-file row element, for ribbon measurement (null on unmount). */
  registerRow: (path: string, el: HTMLElement | null) => void;
  /** Ribbon color per file path, so a node matches the ribbons entering it. */
  colorOf: (path: string) => number | undefined;
  /** Bytes generated per file path, shown as the node's weight. */
  bytesOf: (path: string) => number | undefined;
}

function humanBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  return `${(n / 1024).toFixed(n < 10240 ? 1 : 0)} KB`;
}

function Rows({
  node,
  depth,
  props,
}: {
  node: DirNode;
  depth: number;
  props: OutputTreeProps;
}) {
  const dirs = [...node.dirs.values()].sort((a, b) => a.name.localeCompare(b.name));
  const files = [...node.files].sort((a, b) => a.name.localeCompare(b.name));
  return (
    <>
      {dirs.map((d) => (
        <li key={`d:${d.name}`} className="tree-dir-group">
          <div className="tree-row tree-dir" style={{ paddingLeft: `${depth * 0.75}rem` }}>
            <span className="tree-icon" aria-hidden="true">
              ▾
            </span>
            <span className="tree-name mono">{d.name}/</span>
          </div>
          <ul className="tree-list">
            <Rows node={d} depth={depth + 1} props={props} />
          </ul>
        </li>
      ))}
      {files.map((f) => {
        const active = f.path === props.activePath;
        const color = props.colorOf(f.path);
        const bytes = props.bytesOf(f.path);
        return (
          <li key={`f:${f.path}`}>
            <button
              type="button"
              ref={(el) => props.registerRow(f.path, el)}
              data-path={f.path}
              className={`tree-row tree-file${active ? " on" : ""}${
                color === undefined ? "" : ` tree-c${color}`
              }`}
              style={{ paddingLeft: `${depth * 0.75 + 0.15}rem` }}
              aria-current={active}
              onClick={() => props.onSelect(f.path)}
              data-tip={f.path}
            >
              <span className="tree-name mono">{f.name}</span>
              {bytes !== undefined && <span className="tree-bytes">{humanBytes(bytes)}</span>}
            </button>
          </li>
        );
      })}
    </>
  );
}

export function OutputTree(props: OutputTreeProps) {
  const tree = useMemo(() => buildTree(props.files.map((f) => f.path)), [props.files]);
  return (
    <nav className="output-tree" aria-label="Generated files">
      <h3 className="tree-heading">Generated files</h3>
      <ul className="tree-list tree-root">
        <Rows node={tree} depth={0} props={props} />
      </ul>
    </nav>
  );
}
</book:entry>

</book:doc>
