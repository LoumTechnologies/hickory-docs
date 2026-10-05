declare module "js-interpreter" {
  export default class Interpreter {
    constructor(code: string, init?: (interpreter: Interpreter, global: IObject) => void);
    ast: INode;
    stateStack: IState[];
    globalScope: IScope;
    value: IValue;
    FUNCTION_PROTO: IObject;
    step(): boolean;
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
