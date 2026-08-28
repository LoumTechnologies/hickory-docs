// Reading TypeScript with the compiler, not with a parser.
//
// ts-morph over the TypeScript compiler API, so "is this optional", "what is
// this union made of" and "does this satisfy that interface" are answered by
// the checker. The last one has no syntactic answer at all: TypeScript's
// interfaces are structural, and nothing in the source says who implements
// them.

import { Project, SyntaxKind, ts } from "ts-morph";
import path from "node:path";
import fs from "node:fs";

export class CodeModel {
  constructor(root) {
    this.root = path.resolve(root);
    const config = this.#findConfig();
    this.project = config
      ? new Project({ tsConfigFilePath: config })
      : new Project({ compilerOptions: { strict: true, target: ts.ScriptTarget.ES2022 } });
    if (!config) {
      this.project.addSourceFilesAtPaths([
        `${this.root}/**/*.ts`,
        `${this.root}/**/*.tsx`,
        `!${this.root}/**/node_modules/**`,
        `!${this.root}/**/*.d.ts`,
      ]);
    }
    this.files = this.project
      .getSourceFiles()
      .filter((f) => !f.getFilePath().includes("/node_modules/"));
  }

  #findConfig() {
    const candidate = path.join(this.root, "tsconfig.json");
    return fs.existsSync(candidate) ? candidate : null;
  }

  info() {
    return {
      name: "hick-model-typescript",
      version: "0.1.0",
      language: "typescript",
      root: this.root,
      fileCount: this.files.length,
    };
  }

  /// Diagnostics that mean a TYPE is wrong, which is what makes a model
  /// untrustworthy. A program that fails to build for unrelated reasons still
  /// has a readable model.
  unresolved() {
    return this.project
      .getPreEmitDiagnostics()
      .filter((d) => [2304, 2307, 2503].includes(d.getCode()))
      .slice(0, 50)
      .map((d) => ({
        message: ts.flattenDiagnosticMessageText(d.getMessageText(), " "),
        at: d.getSourceFile() ? this.#spanOfNode(d.getSourceFile(), d.getStart() ?? 0) : null,
      }));
  }

  types({ moduleIs, nameEndsWith, exportedOnly } = {}) {
    if (!this._types) {
      this._types = this.files.flatMap((file) => [
        ...file.getClasses().map((d) => this.#class(d, file)),
        ...file.getInterfaces().map((d) => this.#interface(d, file)),
        ...file.getTypeAliases().map((d) => this.#alias(d, file)),
        ...file.getEnums().map((d) => this.#enum(d, file)),
      ]).sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
    }
    let out = this._types;
    if (moduleIs !== undefined && moduleIs !== null) out = out.filter((t) => t.module === moduleIs);
    if (nameEndsWith) out = out.filter((t) => t.name.endsWith(nameEndsWith));
    if (exportedOnly) out = out.filter((t) => t.isExported);
    return out;
  }

  /// The structural answer. There is no declaration list to read: a class
  /// satisfies an interface by having the right shape, whether or not it says
  /// so, and half the interesting cases in TypeScript never say so.
  implementedBy(interfaceName) {
    const target = this.files
      .flatMap((f) => f.getInterfaces())
      .find((i) => i.getName() === interfaceName);
    if (!target) return [];
    const wanted = target.getType();
    const checker = this.project.getTypeChecker().compilerObject;
    return this.types({}).filter((decl) => {
      if (decl.name === interfaceName) return false;
      const node = decl.__node;
      if (!node || !node.getType) return false;
      try {
        return checker.isTypeAssignableTo(node.getType().compilerType, wanted.compilerType);
      } catch {
        return false;
      }
    });
  }

  /// Every use of a symbol, and who used it.
  ///
  /// `findReferencesAsNodes` is the checker's own answer, so this is not a
  /// text search: a `label` in an unrelated object literal is not a
  /// reference, and one reached through an alias or a re-export is.
  ///
  /// The referring declaration is the field that matters. A generator's
  /// exceptions are written in terms of WHO uses a thing — "except fields the
  /// persistence layer reads" — and a bare file and line cannot answer that.
  references(symbol) {
    const [typeName, member] = symbol.split(".");
    const decl = this.types({}).find((t) => t.name === typeName);
    if (!decl) return [];
    let node = decl.__node;
    if (member) {
      const found =
        node.getProperty?.(member) ?? node.getMethod?.(member) ?? null;
      if (!found) return [];
      node = found;
    }
    if (!node.findReferencesAsNodes) return [];
    return node
      .findReferencesAsNodes()
      .map((ref) => {
        const file = ref.getSourceFile();
        const at = file.getLineAndColumnAtPos(ref.getStart());
        return {
          span: {
            file: path.relative(this.root, file.getFilePath()),
            startLine: at.line, startColumn: at.column,
            endLine: at.line, endColumn: at.column + ref.getWidth(),
          },
          fromModule: path.relative(this.root, file.getFilePath()),
          fromDeclaration: this.#enclosing(ref),
          isWrite: this.#isWrite(ref),
        };
      })
      .sort((a, b) =>
        a.span.file === b.span.file
          ? a.span.startLine - b.span.startLine
          : a.span.file < b.span.file ? -1 : 1);
  }

  /// The named declaration a node sits inside — the thing a rule about layers
  /// actually tests.
  #enclosing(node) {
    let current = node.getParent();
    while (current) {
      const kind = current.getKindName?.();
      if (
        kind === "FunctionDeclaration" || kind === "MethodDeclaration" ||
        kind === "ClassDeclaration" || kind === "InterfaceDeclaration" ||
        kind === "VariableDeclaration" || kind === "PropertyDeclaration"
      ) {
        const owner = current.getFirstAncestorByKind?.(SyntaxKind.ClassDeclaration);
        const name = current.getName?.() ?? "";
        return owner && owner !== current && owner.getName?.()
          ? `${owner.getName()}.${name}` : name;
      }
      current = current.getParent();
    }
    return "";
  }

  /// Left of an assignment. `nothing outside the domain may SET this` needs
  /// reads and writes told apart, which a flat symbol graph cannot do.
  #isWrite(node) {
    const parent = node.getParent();
    if (!parent) return false;
    const binary = parent.getKindName?.() === "BinaryExpression"
      ? parent
      : parent.getParent?.()?.getKindName?.() === "BinaryExpression"
        ? parent.getParent() : null;
    if (!binary) return false;
    const op = binary.getOperatorToken?.().getText?.() ?? "";
    if (!op.endsWith("=") || op === "==" || op === "===" || op === "!=" || op === "!==") {
      return false;
    }
    return binary.getLeft().getStart() <= node.getStart()
        && node.getEnd() <= binary.getLeft().getEnd();
  }

  // ---- declarations ------------------------------------------------------

  #common(node, file, kind) {
    return {
      __typename: kind,
      __node: node,
      name: node.getName?.() ?? "(anonymous)",
      module: path.relative(this.root, file.getFilePath()),
      isExported: node.isExported?.() ?? false,
      declarations: [this.#span(node)],
      properties: this.#properties(node),
      methods: this.#methods(node),
    };
  }

  #class(node, file) {
    return {
      ...this.#common(node, file, "ClassDecl"),
      isAbstract: node.isAbstract(),
      extendsType: node.getExtends() ? this.#typeRefOfNode(node.getExtends()) : null,
      implementsTypes: node.getImplements().map((i) => this.#typeRefOfNode(i)),
      decorators: node.getDecorators().map((d) => this.#decorator(d)),
    };
  }

  #interface(node, file) {
    return {
      ...this.#common(node, file, "InterfaceDecl"),
      extendsTypes: node.getExtends().map((e) => this.#typeRefOfNode(e)),
    };
  }

  #alias(node, file) {
    return {
      ...this.#common(node, file, "TypeAliasDecl"),
      aliasedType: this.#typeRef(node.getType()),
    };
  }

  #enum(node, file) {
    return {
      ...this.#common(node, file, "EnumDecl"),
      isConst: node.isConstEnum(),
      members: node.getMembers().map((m) => ({
        name: m.getName(),
        value: m.getValue() === undefined ? null : String(m.getValue()),
      })),
    };
  }

  #properties(node) {
    if (!node.getProperties) return [];
    return node.getProperties().map((p) => ({
      name: p.getName(),
      type: this.#typeRef(p.getType()),
      accessibility: this.#accessibility(p),
      isStatic: p.isStatic?.() ?? false,
      isReadonly: p.isReadonly?.() ?? false,
      // `?` on the declaration. NOT the same question as whether the type
      // includes undefined, which is why both are exposed.
      isOptional: p.hasQuestionToken?.() ?? false,
      decorators: p.getDecorators?.().map((d) => this.#decorator(d)) ?? [],
      declaration: this.#span(p),
    }));
  }

  #methods(node) {
    if (!node.getMethods) return [];
    return node.getMethods().map((m) => ({
      name: m.getName(),
      returnType: this.#typeRef(m.getReturnType()),
      parameters: (m.getParameters?.() ?? []).map((p) => ({
        name: p.getName(),
        type: this.#typeRef(p.getType()),
        isOptional: p.isOptional?.() ?? false,
        isRest: p.isRestParameter?.() ?? false,
        hasDefault: p.hasInitializer?.() ?? false,
      })),
      accessibility: this.#accessibility(m),
      isStatic: m.isStatic?.() ?? false,
      isAsync: m.isAsync?.() ?? false,
      decorators: m.getDecorators?.().map((d) => this.#decorator(d)) ?? [],
      declaration: this.#span(m),
    }));
  }

  #accessibility(node) {
    // `#name` is a runtime-private JavaScript field and `private` is erased at
    // compile time. Two different things, and a wire generator must not treat
    // them alike.
    if (node.getName?.().startsWith("#")) return "JS_PRIVATE";
    const scope = node.getScope?.();
    if (scope === "private") return "PRIVATE";
    if (scope === "protected") return "PROTECTED";
    return "PUBLIC";
  }

  #decorator(d) {
    return {
      name: d.getName(),
      // Source text, not evaluated values: a decorator's arguments are
      // expressions, and claiming to know what they evaluate to would be a
      // lie the caller cannot check.
      argumentsText: d.getArguments().map((a) => a.getText()),
    };
  }

  #typeRefOfNode(node) {
    return this.#typeRef(node.getType());
  }

  #typeRef(type) {
    const text = type.getText(undefined, ts.TypeFormatFlags.NoTruncation);
    const isUnion = type.isUnion();
    const isIntersection = type.isIntersection();
    const isArray = type.isArray();
    return {
      text,
      kind: isUnion ? "union" : isIntersection ? "intersection"
        : isArray ? "array" : type.isEnum() ? "enum"
        : type.isClassOrInterface() ? "object" : "other",
      isUnion,
      isIntersection,
      members: isUnion ? type.getUnionTypes().map((t) => this.#typeRef(t))
        : isIntersection ? type.getIntersectionTypes().map((t) => this.#typeRef(t))
        : [],
      isArray,
      elementType: isArray && type.getArrayElementType()
        ? this.#typeRef(type.getArrayElementType()) : null,
      isLiteral: type.isLiteral(),
      literalValue: type.isLiteral() ? String(type.getLiteralValue?.() ?? text) : null,
      typeArguments: type.getTypeArguments().map((t) => this.#typeRef(t)),
    };
  }

  #span(node) {
    const file = node.getSourceFile();
    const start = file.getLineAndColumnAtPos(node.getStart());
    const end = file.getLineAndColumnAtPos(node.getEnd());
    return {
      file: path.relative(this.root, file.getFilePath()),
      startLine: start.line, startColumn: start.column,
      endLine: end.line, endColumn: end.column,
    };
  }

  #spanOfNode(file, pos) {
    const at = file.getLineAndColumnAtPos(pos);
    return {
      file: path.relative(this.root, file.getFilePath()),
      startLine: at.line, startColumn: at.column,
      endLine: at.line, endColumn: at.column,
    };
  }
}
