// Parse TypeScript without resolving imports or executing project code.
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
const require = createRequire(new URL('../apps/web/package.json', import.meta.url));
const ts = require('typescript');
const K = ts.SyntaxKind;
const functionKinds = new Set([K.FunctionDeclaration, K.FunctionExpression, K.ArrowFunction,
  K.MethodDeclaration, K.Constructor, K.GetAccessor, K.SetAccessor]);
const decisions = new Set([K.IfStatement, K.ConditionalExpression, K.ForStatement,
  K.ForInStatement, K.ForOfStatement, K.WhileStatement, K.DoStatement,
  K.CaseClause, K.CatchClause]);
const logical = new Set([K.AmpersandAmpersandToken, K.BarBarToken, K.QuestionQuestionToken]);
const literalKinds = new Set([K.StringLiteral, K.NoSubstitutionTemplateLiteral, K.NumericLiteral,
  K.BigIntLiteral, K.TrueKeyword, K.FalseKeyword, K.NullKeyword, K.RegularExpressionLiteral,
  K.TemplateHead, K.TemplateMiddle, K.TemplateTail]);
const callee = node => ts.isIdentifier(node) ? node.text : ts.isPropertyAccessExpression(node)
  ? callee(node.expression) + '.' + node.name.text : '<computed>';

function analyze(input) {
  const file = ts.createSourceFile(input.path, input.code, ts.ScriptTarget.Latest, true,
    input.path.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
  const point = pos => {
    const p = file.getLineAndCharacterOfPosition(pos);
    return { line: p.line, character: p.character };
  };
  const range = node => ({ start: point(node.getStart(file)), end: point(node.end) });
  const errors = file.parseDiagnostics.map(d => ({ message: ts.flattenDiagnosticMessageText(d.messageText, '\n'),
    range: { start: point(d.start ?? 0), end: point((d.start ?? 0) + (d.length ?? 0)) } }));
  if (errors.length) return { path: input.path, errors };
  const units = [], declarationSites = [], imports = new Set();
  function canonical(node, root = false) {
    if (ts.isParenthesizedExpression(node)) return canonical(node.expression, root);
    if (ts.isTypeNode(node) || ts.isTypeParameterDeclaration(node) || ts.isImportDeclaration(node)) return null;
    if (!root && functionKinds.has(node.kind)) return ['nested-function'];
    // Annotations and generic arguments are erased. Keep operators and call targets.
    if (ts.isIdentifier(node)) {
      const p = node.parent;
      if (ts.isCallExpression(p) && p.expression === node) return ['call-name', node.text];
      if (ts.isPropertyAccessExpression(p) && p.name === node) return ['member', node.text];
      return ['identifier'];
    }
    if (literalKinds.has(node.kind)) return ['literal', [K.StringLiteral, K.NoSubstitutionTemplateLiteral].includes(node.kind) ? 'string' : [K.TrueKeyword, K.FalseKeyword].includes(node.kind) ? 'boolean' : K[node.kind]];
    if (ts.isTemplateSpan(node)) return ['template-span', canonical(node.expression)];
    const children = [];
    ts.forEachChild(node, child => { const v = canonical(child); if (v) children.push(v); });
    return [K[node.kind], ...children];
  }
  function measure(node, label, rootNode = node) {
    let nodeCount = 0, maxNesting = 0;
    const sites = [], calls = new Set();
    function walk(n, depth) {
      if (ts.isParenthesizedExpression(n)) { walk(n.expression, depth); return; }
      if (ts.isTypeNode(n) || ts.isTypeParameterDeclaration(n)) return;
      if (n !== rootNode && functionKinds.has(n.kind)) return;
      nodeCount++;
      const branch = decisions.has(n.kind) || (ts.isBinaryExpression(n) && logical.has(n.operatorToken.kind));
      if (branch) { sites.push(range(n)); depth++; maxNesting = Math.max(maxNesting, depth); }
      if (ts.isCallExpression(n) || ts.isNewExpression(n)) calls.add(callee(n.expression));
      ts.forEachChild(n, child => walk(child, depth));
    }
    walk(node, 0);
    return { name: label, range: range(node), node_count: nodeCount,
      decision_sites: sites, max_nesting: maxNesting, calls: [...calls].sort(),
      pattern: canonical(node, true) };
  }
  function visit(node) {
    if (ts.isImportDeclaration(node) && ts.isStringLiteral(node.moduleSpecifier)) imports.add(node.moduleSpecifier.text);
    if (ts.isPropertySignature(node) || ts.isMethodSignature(node) || ts.isTypeAliasDeclaration(node)) {
      declarationSites.push({ kind: ts.isMethodSignature(node) ? 'method' : ts.isPropertySignature(node) ? 'field' : 'alias', range: range(node) });
    }
    if (functionKinds.has(node.kind) && node.body) {
      let label = node.name ? node.name.getText(file) : '<anonymous>';
      if (label === '<anonymous>' && (ts.isPropertyAssignment(node.parent) || ts.isVariableDeclaration(node.parent))) label = node.parent.name.getText(file);
      units.push(measure(node.body, label));
    }
    ts.forEachChild(node, visit);
  }
  visit(file);
  // Top-level runtime statements are a separate unit. Declarations/imports do
  // not duplicate function metrics; initializers containing calls still count.
  const top = file.statements.filter(n => !functionKinds.has(n.kind) && !ts.isClassDeclaration(n)
    && !ts.isInterfaceDeclaration(n) && !ts.isTypeAliasDeclaration(n) && !ts.isImportDeclaration(n)
    && !ts.isExportDeclaration(n));
  if (top.length) {
    const module = measure(file, '<module>');
    // Re-measure only selected top-level statements to exclude declaration syntax.
    const parts = top.map(n => measure(n, '<module>'));
    module.node_count = parts.reduce((s, p) => s + p.node_count, 0);
    module.decision_sites = parts.flatMap(p => p.decision_sites);
    module.max_nesting = Math.max(0, ...parts.map(p => p.max_nesting));
    module.calls = [...new Set(parts.flatMap(p => p.calls))].sort();
    module.pattern = ['module', ...top.map(n => canonical(n))];
    units.push(module);
  }
  return { path: input.path, errors: [], units, declarations: declarationSites, imports: [...imports].sort() };
}
const inputs = JSON.parse(readFileSync(0, 'utf8'));
process.stdout.write(JSON.stringify({ parser: `typescript-${ts.version}`, files: inputs.map(analyze) }));
