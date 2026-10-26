<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Browser debugging runs edited source

For: Authors debugging generated code while navigating its document source.

Given a document containing a literal JavaScript or TypeScript file using the
advertised ES5 function/statement subset,
when the visitor explicitly starts Debug,
then a worker executes that frozen source with a real step-capable interpreter.
Breakpoints stop before executable statements, stepping can enter and leave a
user function, and inspected variables come from live scopes. Editing and
restarting changes the result. A source edit marks a live session as belonging
to the previous revision and removes its marker from the new bytes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/browser-debugging-runs-edited-source.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-7988c63e93d2"></a>
## `apps/web/src/debug/browser/compile.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser debugging runs edited source](contract.md), [Literal files use native bytes](../../embedding/literal-files-use-native-bytes/contract.md).

<a id="f-7988c63e93d2-CompiledProgram"></a>
### `CompiledProgram`

<storycode:copy id="f-7988c63e93d2-CompiledProgram">
import * as ts from "typescript";
import es5Lib from "typescript/lib/lib.es5.d.ts?raw";
import { TraceMap, originalPositionFor } from "@jridgewell/trace-mapping";
import type { LiteralFile } from "../../editor/hickLang";
import { byteToChar, charToByte } from "../../lib/offsets";

export interface CompiledProgram {
  code: string;
  path: string;
  /** Generated JS UTF-16 offset -> frozen document 0-based line. */
  lineAt(offset: number): number | null;
  sourceByteAt(offset: number): number | null;
}

</storycode:copy>
<a id="f-7988c63e93d2-compileProgram"></a>
### `compileProgram`

ES5 and TypeScript type annotations over ES5. No implicit module loader.

<storycode:copy id="f-7988c63e93d2-compileProgram">
/** ES5 and TypeScript type annotations over ES5. No implicit module loader. */
export function compileProgram(file: LiteralFile, source: string): CompiledProgram {
  const typescript = /\.tsx?$/.test(file.path);
  if (!/\.(js|ts)$/.test(file.path)) throw new Error("Browser debugging supports .js and .ts files");
  const tree = ts.createSourceFile(file.path, file.content, ts.ScriptTarget.Latest, true,
    typescript ? ts.ScriptKind.TS : ts.ScriptKind.JS);
  function validate(node: ts.Node) {
    if (ts.isImportDeclaration(node) || ts.isExportDeclaration(node) || ts.isExportAssignment(node)
      || ts.isArrowFunction(node) || ts.isClassDeclaration(node) || ts.isClassExpression(node)
      || ts.isAwaitExpression(node) || ts.isYieldExpression(node) || ts.isForOfStatement(node)
      || ts.isTemplateExpression(node) || ts.isNoSubstitutionTemplateLiteral(node)
      || ts.isSpreadElement(node) || ts.isSpreadAssignment(node)
      || (ts.isArrayBindingPattern(node) || ts.isObjectBindingPattern(node)) || ts.isBigIntLiteral(node)
      || ts.isPropertyAccessExpression(node) && !!node.questionDotToken
      || ts.isElementAccessExpression(node) && !!node.questionDotToken
      || ts.isVariableDeclarationList(node) && !!(node.flags & ts.NodeFlags.BlockScoped)
      || ts.canHaveModifiers(node) && ts.getModifiers(node)?.some((m) => m.kind === ts.SyntaxKind.AsyncKeyword || m.kind === ts.SyntaxKind.ExportKeyword)
      || ts.isCallExpression(node) && (node.expression.kind === ts.SyntaxKind.ImportKeyword || ts.isIdentifier(node.expression) && ["eval", "Function"].includes(node.expression.text))
      || ts.isNewExpression(node) && ts.isIdentifier(node.expression) && node.expression.text === "Function") {
      const line = tree.getLineAndCharacterOfPosition(node.getStart(tree)).line + 1;
      throw new Error(`Unsupported browser syntax on ${file.path}:${line}. Use ES5 statements/functions and var; imports, dynamic code and async are unavailable.`);
    }
    ts.forEachChild(node, validate);
  }
  validate(tree);
  const options: ts.CompilerOptions = { target: ts.ScriptTarget.ES5, module: ts.ModuleKind.None,
    sourceMap: true, strict: true, noEmitOnError: true, noLib: true };
  const declarations = es5Lib + "\ndeclare var console: { log(...values: unknown[]): void; warn(...values: unknown[]): void; error(...values: unknown[]): void; write(...values: unknown[]): void };";
  let code = "", map = "";
  const host: ts.CompilerHost = {
    getSourceFile: (name) => name === file.path ? tree : name === "browser.d.ts" ? ts.createSourceFile(name, declarations, ts.ScriptTarget.ES5, true) : undefined,
    getDefaultLibFileName: () => "browser.d.ts", writeFile: (name, text) => { if (name.endsWith(".map")) map = text; else if (name.endsWith(".js")) code = text; },
    getCurrentDirectory: () => "", getDirectories: () => [], fileExists: (name) => name === file.path || name === "browser.d.ts",
    readFile: (name) => name === file.path ? file.content : name === "browser.d.ts" ? declarations : undefined,
    getCanonicalFileName: (name) => name, useCaseSensitiveFileNames: () => true, getNewLine: () => "\n",
  };
  const program = ts.createProgram([file.path, "browser.d.ts"], options, host);
  const diagnostics = [...program.getSyntacticDiagnostics(), ...(typescript ? program.getSemanticDiagnostics(tree) : [])];
  if (diagnostics.length) throw new Error("Compiler diagnostics: " + diagnostics.map((d) => {
    const line = d.start === undefined ? "" : `${file.path}:${tree.getLineAndCharacterOfPosition(d.start).line + 1}: `;
    return line + ts.flattenDiagnosticMessageText(d.messageText, "\n");
  }).join("\n"));
  if (typescript) program.emit(); else code = file.content;
  const emittedTree = ts.createSourceFile("program.js", code, ts.ScriptTarget.ES5);
  const trace = typescript && map ? new TraceMap(map) : null;
  const sourceByteAt = (offset: number): number | null => {
    let original = offset;
    if (trace) {
      const pos = emittedTree.getLineAndCharacterOfPosition(offset);
      const mapped = originalPositionFor(trace, { line: pos.line + 1, column: pos.character });
      if (mapped.line === null || mapped.column === null) return null;
      original = tree.getPositionOfLineAndCharacter(mapped.line - 1, mapped.column);
    }
    const byte = charToByte(file.content, original);
    const segment = file.segments.find((s) => byte >= s.output[0] && byte < s.output[1]);
    if (!segment) return null;
    const sourceByte = segment.source[0] + byte - segment.output[0];
    return sourceByte;
  };
  return { code, path: file.path, sourceByteAt, lineAt(offset) {
    const byte = sourceByteAt(offset);
    return byte === null ? null : source.slice(0, byteToChar(source, byte)).split("\n").length - 1;
  } };
}
</storycode:copy>

### Assemble `apps/web/src/debug/browser/compile.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/browser/compile.ts"><storycode:paste select="#f-7988c63e93d2-CompiledProgram"/><storycode:paste select="#f-7988c63e93d2-compileProgram"/></storycode:file>

<a id="output-8144eca2a217"></a>
## `apps/web/src/debug/browser/inspect.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser debugging runs edited source](contract.md).

<a id="f-8144eca2a217-preview"></a>
### `preview`

<storycode:copy id="f-8144eca2a217-preview">
import * as ts from "typescript";
import type { IObject, IScope, IValue } from "js-interpreter";
import type { Variable } from "../client";

/** Never call interpreter getProperty/toString: they may execute user getters. */
export function preview(value: IValue): string {
  if (value && typeof value === "object") return value.class === "Function" ? "[Function]" : value.class === "Array" ? `[Array(${value.properties.length ?? "?"})]` : "[Object]";
  if (typeof value === "string") return JSON.stringify(value.slice(0, 1000));
  return String(value);
}

</storycode:copy>
<a id="f-8144eca2a217-Inspector"></a>
### `Inspector`

<storycode:copy id="f-8144eca2a217-Inspector">
export class Inspector {
  private references = new Map<number, IObject>();
  private next = 1;
  reset() { this.references.clear(); this.next = 1; }
  variable(name: string, value: IValue): Variable {
    let reference = 0;
    if (value && typeof value === "object") { reference = this.next++; this.references.set(reference, value); }
    return { name, value: preview(value), type: value === null ? "null" : typeof value, variables_reference: reference };
  }
  children(reference: number): Variable[] {
    const object = this.references.get(reference);
    if (!object) throw new Error("Value belongs to an earlier pause");
    return Object.keys(object.properties).slice(0, 200).map((name) => object.getter?.[name]
      ? { name, value: "[Getter — not invoked]", variables_reference: 0 }
      : this.variable(name, object.properties[name]));
  }
  locals(scope: IScope, globals: Set<string>): Variable[] {
    const seen = new Set<string>();
    const variables: Variable[] = [];
    for (let current: IScope | null = scope; current; current = current.parentScope) {
      for (const name of Object.keys(current.object.properties)) {
        if (seen.has(name) || !current.parentScope && globals.has(name)) continue;
        seen.add(name);
        variables.push(current.object.getter?.[name] ? { name, value: "[Getter — not invoked]", variables_reference: 0 } : this.variable(name, current.object.properties[name]));
        if (variables.length >= 200) return variables;
      }
    }
    return variables;
  }
  evaluate(expression: string, scope: IScope): IValue {
    if (expression.length > 500) throw new Error("Watch expression exceeds 500 characters");
    const tree = ts.createSourceFile("watch.js", `(${expression})`, ts.ScriptTarget.ES5, true, ts.ScriptKind.JS);
    const statement = tree.statements[0];
    if (tree.statements.length !== 1 || !statement || !ts.isExpressionStatement(statement)) throw new Error("Unsupported watch expression");
    let remaining = 100;
    const get = (object: IObject, name: string): IValue => {
      for (let current: IObject | undefined = object; current; current = current.proto) {
        if (current.getter?.[name]) throw new Error("Watch would invoke a getter");
        if (Object.hasOwn(current.properties, name)) return current.properties[name];
      }
      return undefined;
    };
    function primitive(value: IValue): string | number | boolean | null | undefined {
      if (value && typeof value === "object") throw new Error("Watch would coerce an object");
      return value as string | number | boolean | null | undefined;
    }
    const visit = (node: ts.Node): IValue => {
      if (--remaining < 0) throw new Error("Watch expression is too complex");
      if (ts.isParenthesizedExpression(node)) return visit(node.expression);
      if (ts.isNumericLiteral(node)) return Number(node.text);
      if (ts.isStringLiteral(node)) return node.text;
      if (node.kind === ts.SyntaxKind.TrueKeyword) return true;
      if (node.kind === ts.SyntaxKind.FalseKeyword) return false;
      if (node.kind === ts.SyntaxKind.NullKeyword) return null;
      if (ts.isIdentifier(node)) {
        for (let current: IScope | null = scope; current; current = current.parentScope) {
          if (Object.hasOwn(current.object.properties, node.text)) return get(current.object, node.text);
        }
        throw new Error(`No variable named ${node.text}`);
      }
      if (ts.isPropertyAccessExpression(node) || ts.isElementAccessExpression(node)) {
        const object = visit(node.expression);
        if (!object || typeof object !== "object") throw new Error("Watch property access requires an object");
        const name = ts.isPropertyAccessExpression(node) ? node.name.text : String(primitive(visit(node.argumentExpression)));
        return get(object, name);
      }
      if (ts.isPrefixUnaryExpression(node)) {
        const value = primitive(visit(node.operand));
        if (node.operator === ts.SyntaxKind.ExclamationToken) return !value;
        if (node.operator === ts.SyntaxKind.MinusToken) return -Number(value);
        if (node.operator === ts.SyntaxKind.PlusToken) return Number(value);
      }
      if (ts.isBinaryExpression(node)) {
        const left = primitive(visit(node.left));
        const op = node.operatorToken.kind;
        if (op === ts.SyntaxKind.AmpersandAmpersandToken) return left ? visit(node.right) : left;
        if (op === ts.SyntaxKind.BarBarToken) return left || visit(node.right);
        const right = primitive(visit(node.right));
        switch (op) {
          case ts.SyntaxKind.PlusToken: return typeof left === "string" || typeof right === "string" ? String(left) + String(right) : Number(left) + Number(right);
          case ts.SyntaxKind.MinusToken: return Number(left) - Number(right);
          case ts.SyntaxKind.AsteriskToken: return Number(left) * Number(right);
          case ts.SyntaxKind.SlashToken: return Number(left) / Number(right);
          case ts.SyntaxKind.PercentToken: return Number(left) % Number(right);
          case ts.SyntaxKind.EqualsEqualsEqualsToken: return left === right;
          case ts.SyntaxKind.ExclamationEqualsEqualsToken: return left !== right;
          case ts.SyntaxKind.LessThanToken: return Number(left) < Number(right);
          case ts.SyntaxKind.GreaterThanToken: return Number(left) > Number(right);
        }
      }
      throw new Error("Unsupported watch expression: calls, assignments and getters are unavailable");
    };
    return visit(statement.expression);
  }
}
</storycode:copy>

### Assemble `apps/web/src/debug/browser/inspect.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/browser/inspect.ts"><storycode:paste select="#f-8144eca2a217-preview"/><storycode:paste select="#f-8144eca2a217-Inspector"/></storycode:file>

<a id="output-eded76a6db69"></a>
## `apps/web/src/debug/browser/worker.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser debugging runs edited source](contract.md).

<a id="f-eded76a6db69-run"></a>
### `run`

<storycode:copy id="f-eded76a6db69-run">
import { BrowserRun } from "./runtime";
import type { BrowserInput } from "./runtime";
import type { DebugBreakpoint, Step } from "../client";

let run: BrowserRun | null = null;
</storycode:copy>
<a id="f-eded76a6db69-timer"></a>
### `timer`

<storycode:copy id="f-eded76a6db69-timer">
let timer: ReturnType<typeof setTimeout> | null = null;
</storycode:copy>
<a id="f-eded76a6db69-schedule"></a>
### `schedule`

<storycode:copy id="f-eded76a6db69-schedule">
function schedule() {
  if (timer !== null) return;
  timer = setTimeout(() => { timer = null; if (run?.pump()) schedule(); }, 0);
}
</storycode:copy>
<a id="f-eded76a6db69-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-eded76a6db69-expression_statement">
globalThis.onmessage = (event: MessageEvent<Record<string, unknown> & { input: BrowserInput }>) => {
  const request = event.data;
  try {
    if (request.op === "start") {
      run?.stop();
      run = new BrowserRun(request.input, String(request.session), String(request.doc), request.breakpoints as DebugBreakpoint[], (e) => globalThis.postMessage(e));
      schedule();
      return;
    }
    if (!run || request.session !== run.session) throw new Error("Unknown browser session");
    switch (request.op) {
      case "step": run.step(request.how as Step); schedule(); break;
      case "pause": run.pause(); schedule(); break;
      case "state": run.report("frame", request.frame as number | undefined); break;
      case "breakpoints": run.setBreakpoints(request.breakpoints as DebugBreakpoint[]); break;
      case "eval": run.evaluate(String(request.expression), request.frame as number | undefined); break;
      case "children": run.children(Number(request.reference)); break;
      default: throw new Error(`Unsupported browser debug action: ${request.op}`);
    }
  } catch (e) {
    globalThis.postMessage({ event: "failed", session: run?.session ?? null, doc: request.doc, about: request.op,
      message: e instanceof Error ? e.message : String(e) });
  }
};
</storycode:copy>

### Assemble `apps/web/src/debug/browser/worker.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/browser/worker.ts"><storycode:paste select="#f-eded76a6db69-run"/><storycode:paste select="#f-eded76a6db69-timer"/><storycode:paste select="#f-eded76a6db69-schedule"/><storycode:paste select="#f-eded76a6db69-expression_statement"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/debug/browser/compile.ts](contract.md#output-7988c63e93d2)
- [apps/web/src/debug/browser/inspect.ts](contract.md#output-8144eca2a217)
- [apps/web/src/debug/browser/runtime.ts](../../../mechanisms/apps/web/src/debug/browser/runtime.ts.md#output-6c24fa24bf81)
- [apps/web/src/debug/browser/transport.ts](../../../mechanisms/apps/web/src/debug/browser/transport.ts.md#output-4357b3feff1c)
- [apps/web/src/debug/browser/worker.ts](contract.md#output-eded76a6db69)
- [apps/web/src/embed/DebuggableDocument.tsx](../../../mechanisms/apps/web/src/embed/DebuggableDocument.tsx.md#output-b948f7ea7230)
- [apps/web/src/landing/demos/BrowserDebugDemo.tsx](../../../mechanisms/apps/web/src/landing/demos/BrowserDebugDemo.tsx.md#output-8701a828ceab)

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/debug/browser/runtime.test.ts](../../../mechanisms/apps/web/src/debug/browser/runtime.ts.md#output-de529cc60d5d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
