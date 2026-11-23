<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
# Architecture from the TypeScript code model

Each box opens a TypeScript module. An arrow means the compiler resolved a
reference in one module to a declaration in another. Save a change to the
modules and the open diagram updates automatically.

Use **View source** to see the two functions that produce the graph. Change
the GraphQL selection to ask the code model for different facts, or change
the graph builder to group and label those facts differently.

<h:diagram renderer="code-model">
function codeModelQuery() {
  return {
    language: "typescript",
    query: `{
      sourceIndex {
        files { path occurrences { symbols unresolved } }
        symbols { id declarations { file } }
      }
    }`,
  };
}

function buildDiagram({ sourceIndex }) {
  const files = sourceIndex.files.filter(file => /^shop\/[^/]+\.ts$/.test(file.path));
  const visible = new Set(files.map(file => file.path));
  const declarations = new Map(sourceIndex.symbols.map(symbol => [symbol.id, symbol.declarations]));

  // Node destinations are paths relative to this document, with one-based lines.
  const nodes = files.map(({ path }) => ({
    id: path,
    label: path.split("/").pop(),
    href: path.split("/").map(encodeURIComponent).join("/") + "#L1",
  }));

  // An import and its uses can refer to the same module. Draw one arrow per pair.
  const edges = new Map();
  for (const file of files) {
    for (const occurrence of file.occurrences) {
      if (occurrence.unresolved) continue;
      for (const symbol of occurrence.symbols) {
        for (const declaration of declarations.get(symbol) ?? []) {
          if (visible.has(declaration.file) && declaration.file !== file.path) {
            edges.set(JSON.stringify([file.path, declaration.file]), {
              from: file.path,
              to: declaration.file,
            });
          }
        }
      }
    }
  }
  return { topology: { nodes, edges: Array.from(edges.values()) } };
}
</h:diagram>
</h:doc>
