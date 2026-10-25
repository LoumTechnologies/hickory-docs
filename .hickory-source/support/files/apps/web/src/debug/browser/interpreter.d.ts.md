<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/browser/interpreter.d.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `6631f292119fa9d8806eab900f5e4c9b7a79ce58365c0636d8a2eb7677267b88`.

<book:entry path="apps/web/src/debug/browser/interpreter.d.ts">
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
</book:entry>

</book:doc>
