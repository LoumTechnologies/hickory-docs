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
}

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
  const declarations = es5Lib + "\ndeclare var console: { log(...values: unknown[]): void; warn(...values: unknown[]): void; error(...values: unknown[]): void };";
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
  return { code, path: file.path, lineAt(offset) {
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
    return source.slice(0, byteToChar(source, sourceByte)).split("\n").length - 1;
  } };
}
