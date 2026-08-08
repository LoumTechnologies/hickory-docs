// Collapsible folder tree for a project's documents. Docs have no folder
// entity of their own — hierarchy comes straight from `path` segments, the
// same way OutputTree groups generated files. See buildTree in ../lib/tree.

import { useMemo, useState } from "react";
import type { DocSummary } from "../api/types";
import { buildTree, type DirNode } from "../lib/tree";

export interface ProjectDocTreeProps {
  docs: DocSummary[];
  activeDocId: string | null;
  onSelect: (id: string) => void;
  /** Start the create-doc form scoped under this folder ("" = project root). */
  onCreate: (folderPath: string) => void;
}

function Rows({
  node,
  folderPath,
  depth,
  collapsed,
  toggle,
  byPath,
  props,
}: {
  node: DirNode;
  folderPath: string;
  depth: number;
  collapsed: Set<string>;
  toggle: (path: string) => void;
  byPath: Map<string, DocSummary>;
  props: ProjectDocTreeProps;
}) {
  const dirs = [...node.dirs.values()].sort((a, b) => a.name.localeCompare(b.name));
  const files = [...node.files].sort((a, b) => a.name.localeCompare(b.name));
  return (
    <>
      {dirs.map((d) => {
        const path = folderPath ? `${folderPath}/${d.name}` : d.name;
        const isCollapsed = collapsed.has(path);
        return (
          <li key={`d:${path}`} className="tree-dir-group">
            <div className="tree-row tree-dir" style={{ paddingLeft: `${depth * 0.75}rem` }}>
              <button
                type="button"
                className="tree-toggle"
                onClick={() => toggle(path)}
                aria-label={isCollapsed ? "Expand folder" : "Collapse folder"}
              >
                <span className="tree-icon" aria-hidden="true">
                  {isCollapsed ? "▸" : "▾"}
                </span>
                <span className="tree-name mono">{d.name}/</span>
              </button>
              <button
                type="button"
                className="tree-add-btn"
                title={`New document in ${path}/`}
                onClick={() => props.onCreate(path)}
              >
                +
              </button>
            </div>
            {!isCollapsed && (
              <ul className="tree-list">
                <Rows
                  node={d}
                  folderPath={path}
                  depth={depth + 1}
                  collapsed={collapsed}
                  toggle={toggle}
                  byPath={byPath}
                  props={props}
                />
              </ul>
            )}
          </li>
        );
      })}
      {files.map((f) => {
        const doc = byPath.get(f.path);
        if (!doc) return null;
        const active = doc.id === props.activeDocId;
        return (
          <li key={`f:${f.path}`}>
            <button
              type="button"
              className={`tree-row tree-file${active ? " on" : ""}`}
              style={{ paddingLeft: `${depth * 0.75 + 0.15}rem` }}
              aria-current={active}
              onClick={() => props.onSelect(doc.id)}
              title={f.path}
            >
              <span className="tree-name mono">{f.name}</span>
              <span className="tree-bytes muted">
                {new Date(doc.updated_at).toLocaleDateString()}
              </span>
            </button>
          </li>
        );
      })}
    </>
  );
}

export function ProjectDocTree(props: ProjectDocTreeProps) {
  const tree = useMemo(() => buildTree(props.docs.map((d) => d.path)), [props.docs]);
  const byPath = useMemo(() => new Map(props.docs.map((d) => [d.path, d])), [props.docs]);
  const [collapsed, setCollapsed] = useState<Set<string>>(() => new Set());
  const toggle = (path: string) =>
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });

  return (
    <div className="doc-tree">
      <div className="tree-row tree-root-row">
        <span className="tree-heading">Documents</span>
        <button type="button" className="tree-add-btn" title="New document" onClick={() => props.onCreate("")}>
          +
        </button>
      </div>
      <ul className="tree-list tree-root">
        <Rows
          node={tree}
          folderPath=""
          depth={0}
          collapsed={collapsed}
          toggle={toggle}
          byPath={byPath}
          props={props}
        />
      </ul>
    </div>
  );
}
