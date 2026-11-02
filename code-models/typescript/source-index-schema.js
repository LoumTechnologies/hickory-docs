import { GraphQLObjectType, GraphQLString, GraphQLInt, GraphQLBoolean, GraphQLList, GraphQLNonNull } from "graphql";
const list = type => new GraphQLNonNull(new GraphQLList(new GraphQLNonNull(type)));
const field = type => ({ type: new GraphQLNonNull(type) });
const object = (name, fields) => new GraphQLObjectType({ name, fields });
const Location = object("TypeScriptSourceLocation", { file: field(GraphQLString), from: field(GraphQLInt), to: field(GraphQLInt) });
const Symbol = object("TypeScriptIndexedSymbol", { id: field(GraphQLString), name: field(GraphQLString), kind: field(GraphQLString), documentation: field(GraphQLString), declarations: { type: list(Location) } });
const Occurrence = object("TypeScriptOccurrence", {
  from: field(GraphQLInt), to: field(GraphQLInt), symbols: { type: list(GraphQLString) }, type: field(GraphQLString),
  isDefinition: field(GraphQLBoolean), isWrite: field(GraphQLBoolean), unresolved: field(GraphQLBoolean), external: field(GraphQLBoolean),
});
const Call = object("TypeScriptIndexedCall", { from: field(GraphQLInt), to: field(GraphQLInt), signature: field(GraphQLString) });
const Fold = object("TypeScriptIndexedFold", { from: field(GraphQLInt), to: field(GraphQLInt), kind: field(GraphQLString) });
const Hint = object("TypeScriptIndexedHint", { position: field(GraphQLInt), text: field(GraphQLString), kind: field(GraphQLString), whitespaceBefore: field(GraphQLBoolean), whitespaceAfter: field(GraphQLBoolean) });
const File = object("TypeScriptIndexedFile", {
  path: field(GraphQLString), sha256: field(GraphQLString), length: field(GraphQLInt), occurrences: { type: list(Occurrence) },
  calls: { type: list(Call) }, folds: { type: list(Fold) }, classifications: { type: list(GraphQLInt) }, hints: { type: list(Hint) },
});
const Diagnostic = object("TypeScriptIndexedDiagnostic", { file: { type: GraphQLString }, from: field(GraphQLInt), to: field(GraphQLInt), code: field(GraphQLInt), category: field(GraphQLInt), message: field(GraphQLString) });
export const SourceIndex = object("TypeScriptSourceIndex", {
  version: field(GraphQLInt), coordinateEncoding: field(GraphQLString), files: { type: list(File) },
  symbols: { type: list(Symbol) }, diagnostics: { type: list(Diagnostic) },
});
