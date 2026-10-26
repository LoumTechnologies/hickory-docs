<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/browser/interpreter.d.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-31d084b7b65a"></a>
## `apps/web/src/debug/browser/interpreter.d.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-31d084b7b65a-ambient_declaration"></a>
### `ambient_declaration`

<storycode:copy id="f-31d084b7b65a-ambient_declaration">
declare module "js-interpreter" {
  export default class Interpreter {
    constructor(code: string, init?: (interpreter: Interpreter, global: IObject) => void);
    ast: INode;
    stateStack: IState[];
    globalScope: IScope;
    value: IValue;
    FUNCTION_PROTO: IObject;
    step(): boolean;
    throwException(value: IValue, message?: string): void;
    createObjectProto(proto: IObject | null): IObject;
    createNativeFunction(fn: (...args: IValue[]) => IValue): IObject;
    setProperty(object: IObject, name: string, value: IValue): void;
  }
  export type IValue = undefined | null | string | number | boolean | IObject;
  export interface IObject {
    properties: Record<string, IValue>;
    getter: Record<string, IValue>;
    proto?: IObject;
    class?: string;
    node?: INode;
  }
  export interface IScope { object: IObject; parentScope: IScope | null; }
  export interface INode {
    type: string;
    start: number;
    end: number;
    id?: { name: string };
    loc?: { start: { line: number; column: number } };
    [key: string]: unknown;
  }
  export interface IState { node: INode; scope: IScope; func_?: IObject; doneExec_?: boolean; }
}
</storycode:copy>

### Assemble `apps/web/src/debug/browser/interpreter.d.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/browser/interpreter.d.ts"><storycode:paste select="#f-31d084b7b65a-ambient_declaration"/></storycode:file>

</storycode:doc>
