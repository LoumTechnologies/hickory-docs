<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain code-models/typescript/server.js

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-cc99137d06f9"></a>
## `code-models/typescript/server.js`

Expose compiler-backed TypeScript structure to read-only generators; language semantics and unresolved references remain explicit.

<a id="f-cc99137d06f9-hash_bang_line"></a>
### `hash_bang_line`

<storycode:copy id="f-cc99137d06f9-hash_bang_line">
#!/usr/bin/env node
</storycode:copy>
<a id="f-cc99137d06f9-import_statement"></a>
### `import_statement`

`hick-model-typescript <root>` — the TypeScript code model, served as GraphQL.
One JSON object per line on stdin, one per line on stdout. Diagnostics go to
stderr; stdout carries the protocol and nothing else.

<storycode:copy id="f-cc99137d06f9-import_statement">
// `hick-model-typescript <root>` — the TypeScript code model, served as GraphQL.
//
// One JSON object per line on stdin, one per line on stdout. Diagnostics go to
// stderr; stdout carries the protocol and nothing else.

import { graphql } from "graphql";
</storycode:copy>
<a id="f-cc99137d06f9-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-cc99137d06f9-import_statement-2">
import readline from "node:readline";
</storycode:copy>
<a id="f-cc99137d06f9-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-cc99137d06f9-import_statement-3">
import fs from "node:fs";
</storycode:copy>
<a id="f-cc99137d06f9-import_statement-4"></a>
### `import_statement`

<storycode:copy id="f-cc99137d06f9-import_statement-4">
import { CodeModel } from "./model.js";
</storycode:copy>
<a id="f-cc99137d06f9-import_statement-5"></a>
### `import_statement`

<storycode:copy id="f-cc99137d06f9-import_statement-5">
import { buildSchema } from "./schema.js";

</storycode:copy>
<a id="f-cc99137d06f9-root"></a>
### `root`

<storycode:copy id="f-cc99137d06f9-root">
const root = process.argv[2] ?? process.cwd();
</storycode:copy>
<a id="f-cc99137d06f9-if_statement"></a>
### `if_statement`

<storycode:copy id="f-cc99137d06f9-if_statement">
if (!fs.existsSync(root) || !fs.statSync(root).isDirectory()) {
  console.error(`hick-model-typescript: no such directory: ${root}`);
  console.error("  Pass the source root to model, e.g. `hick-model-typescript src`.");
  process.exit(2);
}

</storycode:copy>
<a id="f-cc99137d06f9-model"></a>
### `model`

<storycode:copy id="f-cc99137d06f9-model">
let model;
</storycode:copy>
<a id="f-cc99137d06f9-try_statement"></a>
### `try_statement`

<storycode:copy id="f-cc99137d06f9-try_statement">
try {
  model = new CodeModel(root);
} catch (e) {
  console.error(`hick-model-typescript: could not read ${root}: ${e.message}`);
  process.exit(2);
}
</storycode:copy>
<a id="f-cc99137d06f9-schema"></a>
### `schema`

<storycode:copy id="f-cc99137d06f9-schema">
const schema = buildSchema(model);
</storycode:copy>
<a id="f-cc99137d06f9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-cc99137d06f9-expression_statement">
console.error(`hick-model-typescript: ${model.info().fileCount} file(s) under ${model.info().root}; ready`);
</storycode:copy>
<a id="f-cc99137d06f9-for_in_statement"></a>
### `for_in_statement`

<storycode:copy id="f-cc99137d06f9-for_in_statement">
for (const gap of model.unresolved().slice(0, 5)) console.error(`  unresolved: ${gap.message}`);

</storycode:copy>
<a id="f-cc99137d06f9-lines"></a>
### `lines`

<storycode:copy id="f-cc99137d06f9-lines">
const lines = readline.createInterface({ input: process.stdin, terminal: false });
</storycode:copy>
<a id="f-cc99137d06f9-for_in_statement-2"></a>
### `for_in_statement`

<storycode:copy id="f-cc99137d06f9-for_in_statement-2">
for await (const line of lines) {
  if (!line.trim()) continue;
  let body;
  try {
    const request = JSON.parse(line);
    const result = await graphql({
      schema,
      source: request.query ?? "",
      variableValues: request.variables ?? undefined,
    });
    body = JSON.stringify(result);
  } catch (e) {
    // A malformed request must not end the session: the caller is a generator
    // being written, and being written means getting it wrong.
    body = JSON.stringify({ errors: [{ message: e.message }] });
  }
  process.stdout.write(body.replace(/\n/g, " ") + "\n");
}
</storycode:copy>

### Assemble `code-models/typescript/server.js`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="code-models/typescript/server.js" executable="true"><storycode:paste select="#f-cc99137d06f9-hash_bang_line"/><storycode:paste select="#f-cc99137d06f9-import_statement"/><storycode:paste select="#f-cc99137d06f9-import_statement-2"/><storycode:paste select="#f-cc99137d06f9-import_statement-3"/><storycode:paste select="#f-cc99137d06f9-import_statement-4"/><storycode:paste select="#f-cc99137d06f9-import_statement-5"/><storycode:paste select="#f-cc99137d06f9-root"/><storycode:paste select="#f-cc99137d06f9-if_statement"/><storycode:paste select="#f-cc99137d06f9-model"/><storycode:paste select="#f-cc99137d06f9-try_statement"/><storycode:paste select="#f-cc99137d06f9-schema"/><storycode:paste select="#f-cc99137d06f9-expression_statement"/><storycode:paste select="#f-cc99137d06f9-for_in_statement"/><storycode:paste select="#f-cc99137d06f9-lines"/><storycode:paste select="#f-cc99137d06f9-for_in_statement-2"/></storycode:file>

</storycode:doc>
