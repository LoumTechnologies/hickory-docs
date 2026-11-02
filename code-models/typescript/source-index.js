import { ts } from "ts-morph";
import path from "node:path";
import crypto from "node:crypto";
import fs from "node:fs";

export function buildSourceIndex(model) {
  const checker = model.project.getTypeChecker().compilerObject;
  const service = model.project.getLanguageService().compilerObject;
  const selected = new Set(model.files.map(file => file.getFilePath()));
  const contents = new Map(model.files.map(source => {
    const file = source.compilerNode, text = fs.readFileSync(file.fileName, "utf8");
    const shift = text.startsWith("\ufeff") && file.text === text.slice(1) ? 1 : 0;
    if (file.text !== text.slice(shift)) throw Error("Source changed during model capture: " + file.fileName);
    return [file.fileName, { text, shift }];
  }));
  const relative = file => path.relative(model.root, file).replaceAll(path.sep, "/");
  const location = node => {
    const file = node.getSourceFile();
    if (!selected.has(file.fileName)) return null;
    const shift = contents.get(file.fileName).shift;
    return { file: relative(file.fileName), from: node.getStart(file)+shift, to: node.getEnd()+shift };
  };
  const roots = symbol => {
    const seen = new Set();
    const visit = value => {
      if (!value || seen.has(value)) return [];
      seen.add(value);
      if (value.flags & ts.SymbolFlags.Alias) {
        const target = checker.getAliasedSymbol(value);
        if (target !== value) return visit(target);
      }
      const originals = checker.getRootSymbols(value);
      if (originals.length && (originals.length !== 1 || originals[0] !== value)) return originals.flatMap(visit);
      return [value];
    };
    return visit(symbol);
  };
  const symbols = new Map(), identities = new Map();
  const describe = symbol => {
    if (identities.has(symbol)) return identities.get(symbol);
    const declarations = (symbol.declarations ?? []).map(node => location(node.name ?? node)).filter(Boolean)
      .sort((a, b) => a.file.localeCompare(b.file) || a.from - b.from || a.to - b.to);
    if (!declarations.length) { identities.set(symbol, null); return null; }
    const named = symbol.declarations?.find(node => node.name && (ts.isPrivateIdentifier(node.name) || ts.isComputedPropertyName(node.name)))?.name;
    const name = named ? named.getText() : symbol.getName();
    const id = crypto.createHash("sha256").update(JSON.stringify([name, declarations])).digest("hex");
    identities.set(symbol, id);
    const kinds = [[ts.SymbolFlags.Class, "class"], [ts.SymbolFlags.Interface, "interface"], [ts.SymbolFlags.Enum, "enum"], [ts.SymbolFlags.TypeAlias, "type"], [ts.SymbolFlags.TypeParameter, "typeParameter"], [ts.SymbolFlags.Method, "method"], [ts.SymbolFlags.Property, "property"], [ts.SymbolFlags.Function, "function"], [ts.SymbolFlags.NamespaceModule, "namespace"]];
    const kind = kinds.find(([flag]) => symbol.flags & flag)?.[1] ?? "variable";
    if (!symbols.has(id)) symbols.set(id, { id, name, kind, documentation: ts.displayPartsToString(symbol.getDocumentationComment(checker)), declarations });
    return id;
  };
  const contextualProperties = node => {
    const parent = node.parent;
    if (!(ts.isPropertyAssignment(parent) || ts.isShorthandPropertyAssignment(parent) || ts.isMethodDeclaration(parent)) || parent.name !== node || !ts.isObjectLiteralExpression(parent.parent)) return [];
    const contextual = checker.getContextualType(parent.parent);
    if (!contextual) return [];
    const types = contextual.isUnionOrIntersection() ? contextual.types : [contextual];
    return types.map(type => checker.getPropertyOfType(type, node.text)).filter(Boolean);
  };
  const nodeSymbols = node => {
    const contextual = contextualProperties(node);
    const raw = checker.getSymbolAtLocation(node);
    const values = contextual.length ? contextual : raw ? [raw] : [];
    if (ts.isShorthandPropertyAssignment(node.parent)) {
      const value = checker.getShorthandAssignmentValueSymbol(node.parent);
      if (value) values.push(value);
    }
    return [...new Set(values.flatMap(roots).map(describe).filter(Boolean))].sort();
  };
  const isWrite = node => {
    let target = node;
    if (ts.isPropertyAccessExpression(node.parent) && node.parent.name === node) target = node.parent;
    const parent = target.parent;
    return ts.isDeclarationName(node) ||
      (ts.isBinaryExpression(parent) && parent.left === target && ts.isAssignmentOperator(parent.operatorToken.kind)) ||
      ((ts.isPrefixUnaryExpression(parent) || ts.isPostfixUnaryExpression(parent)) &&
        [ts.SyntaxKind.PlusPlusToken, ts.SyntaxKind.MinusMinusToken].includes(parent.operator));
  };
  const files = model.files.slice().sort((a, b) => a.getFilePath().localeCompare(b.getFilePath())).map(source => {
    const file = source.compilerNode, occurrences = [], calls = [], folds = [];
    const { text, shift } = contents.get(file.fileName);
    const visit = node => {
      if (ts.isIdentifier(node) || ts.isPrivateIdentifier(node) ||
          ((ts.isStringLiteral(node) || ts.isNumericLiteral(node)) &&
           (node.parent.name === node || ts.isElementAccessExpression(node.parent)))) {
        const ids = nodeSymbols(node);
        const raw = checker.getSymbolAtLocation(node);
        const external = ids.length === 0 && roots(raw).some(symbol => symbol.declarations?.length);
        const type = checker.getTypeAtLocation(node);
        occurrences.push({ from: node.getStart(file)+shift, to: node.getEnd()+shift, symbols: ids,
          type: checker.typeToString(type, node, ts.TypeFormatFlags.UseAliasDefinedOutsideCurrentScope),
          isDefinition: ts.isDeclarationName(node), isWrite: !!isWrite(node), unresolved: ids.length === 0 && !external, external });
      }
      if (ts.isCallExpression(node) || ts.isNewExpression(node)) {
        const signature = checker.getResolvedSignature(node);
        if (signature) calls.push({ from: node.getStart(file)+shift, to: node.getEnd()+shift, signature: checker.signatureToString(signature, node, ts.TypeFormatFlags.UseAliasDefinedOutsideCurrentScope) });
      }
      ts.forEachChild(node, visit);
    };
    visit(file);
    for (const item of service.getOutliningSpans(file.fileName)) folds.push({ from: item.textSpan.start+shift, to: item.textSpan.start + item.textSpan.length+shift, kind: item.kind });
    const classifications = service.getEncodedSemanticClassifications(file.fileName, { start: 0, length: file.text.length }, ts.SemanticClassificationFormat.TwentyTwenty).spans.map((value, i) => i % 3 === 0 ? value+shift : value);
    const hints = service.provideInlayHints(file.fileName, { start: 0, length: file.text.length }, {
      includeInlayParameterNameHints: "all", includeInlayParameterNameHintsWhenArgumentMatchesName: false,
      includeInlayVariableTypeHints: true, includeInlayPropertyDeclarationTypeHints: true,
      includeInlayFunctionLikeReturnTypeHints: true, includeInlayEnumMemberValueHints: true,
    }).map(hint => ({ position: hint.position+shift, text: hint.text, kind: hint.kind, whitespaceBefore: !!hint.whitespaceBefore, whitespaceAfter: !!hint.whitespaceAfter }));
    return { path: relative(file.fileName), sha256: crypto.createHash("sha256").update(text).digest("hex"), length: text.length, occurrences, calls, folds, classifications, hints };
  });
  const diagnostics = model.project.getPreEmitDiagnostics().map(item => {
    const diagnostic = item.compilerObject;
    const shift = diagnostic.file ? contents.get(diagnostic.file.fileName)?.shift ?? 0 : 0;
    return { file: diagnostic.file && selected.has(diagnostic.file.fileName) ? relative(diagnostic.file.fileName) : null,
      from: (diagnostic.start ?? 0)+shift, to: (diagnostic.start ?? 0) + (diagnostic.length ?? 0)+shift, code: diagnostic.code,
      category: diagnostic.category, message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n") };
  });
  return { version: 1, coordinateEncoding: "utf-16", files, symbols: [...symbols.values()].sort((a, b) => a.id.localeCompare(b.id)), diagnostics };
}
