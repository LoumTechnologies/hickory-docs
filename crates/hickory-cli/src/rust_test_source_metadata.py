import json
import os

query = json.loads(os.environ["HICKORY_RUST_META_QUERY"])
if set(query) != {"base", "first", "last", "image"} or not isinstance(query["base"], str):
    raise ValueError("Malformed source metadata query")
if type(query["first"]) is not int or type(query["last"]) is not int or not 1 <= query["first"] <= query["last"] <= 2**32 - 2:
    raise ValueError("Invalid source metadata range")
target = lldb.debugger.GetSelectedTarget()
spec = target.GetExecutable()
image = os.path.realpath(os.path.join(spec.GetDirectory() or "", spec.GetFilename() or ""))
if image != query["image"] or target.GetProcess().IsValid():
    raise ValueError("Unexpected image or process during read-only metadata inspection")
main = target.FindModule(spec)
if not main.IsValid():
    raise ValueError("The admitted image has no readable module")
contexts = target.FindFunctions(query["base"], lldb.eFunctionNameTypeBase)
if contexts.GetSize() > 1024:
    raise ValueError("Source metadata symbol budget exceeded")
rows = []
work = 0
for context in contexts:
    if context.GetModule() != main:
        continue
    function = context.GetFunction()
    if not function.IsValid():
        symbol = context.GetSymbol()
        if symbol.IsValid() and symbol.GetName():
            rows.append({"name": symbol.GetName(), "optimized": True, "entries": []})
        continue
    start = function.GetStartAddress()
    end = function.GetEndAddress().GetFileAddress()
    unit = context.GetCompileUnit()
    entry = start.GetLineEntry()
    entries = []
    index = unit.FindLineEntryIndex(entry, True) if unit.IsValid() and entry.IsValid() else 2**32 - 1
    if index != 2**32 - 1:
        while index < unit.GetNumLineEntries():
            work += 1
            if work > 100000:
                raise ValueError("Source metadata line budget exceeded")
            value = unit.GetLineEntryAtIndex(index)
            address = value.GetStartAddress().GetFileAddress()
            if address >= end:
                break
            position = value.GetStartAddress()
            block = position.GetBlock()
            own_source = position.GetFunction() == function and block.IsValid() and not block.IsInlined() and not block.GetContainingInlinedBlock().IsValid()
            if own_source and 0 < value.GetLine() < 2**32 - 1 and (not entries or query["first"] <= value.GetLine() <= query["last"]):
                spec = value.GetFileSpec()
                row = {"file": os.path.normpath(os.path.join(spec.GetDirectory() or "", spec.GetFilename() or "")), "line": value.GetLine(), "column": value.GetColumn(), "address": str(address)}
                if row not in entries:
                    entries.append(row)
            if len(entries) > 4096:
                raise ValueError("Source metadata entry budget exceeded")
            index += 1
    rows.append({"name": function.GetName(), "optimized": function.GetIsOptimized(), "entries": entries})
print("HICKORY_RUST_SOURCE_V1=" + json.dumps({"version": 1, "producer": "lldb-image-source-v1", "functions": rows}, ensure_ascii=True))
