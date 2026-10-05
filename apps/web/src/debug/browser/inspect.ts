import * as ts from "typescript";
import type { IObject, IScope, IValue } from "js-interpreter";
import type { Variable } from "../client";

/** Never call interpreter getProperty/toString: they may execute user getters. */
export function preview(value: IValue): string {
  if (value && typeof value === "object") return value.class === "Function" ? "[Function]" : value.class === "Array" ? `[Array(${value.properties.length ?? "?"})]` : "[Object]";
  if (typeof value === "string") return JSON.stringify(value.slice(0, 1000));
  return String(value);
}

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
