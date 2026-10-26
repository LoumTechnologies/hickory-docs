<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/python_observer.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-01359553f977"></a>
## `crates/hick-plugins/src/languages/python_observer.py`

<a id="f-01359553f977-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement">
"""Reviewed CPython hooks, run only inside the existing cell sandbox.
Not a security/attestation boundary against code modifying its interpreter.
"""
</storycode:copy>
<a id="f-01359553f977-sys"></a>
### `sys`

<storycode:copy id="f-01359553f977-sys">
import sys, os, json, time, threading, asyncio, traceback, runpy, builtins, cProfile, pstats, platform
</storycode:copy>
<a id="f-01359553f977-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-2">
ROOT = os.getcwd()
</storycode:copy>
<a id="f-01359553f977-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-3">
OUT = open("observations.jsonl", "w", encoding="utf-8")
</storycode:copy>
<a id="f-01359553f977-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-4">
LOCK = threading.RLock()
</storycode:copy>
<a id="f-01359553f977-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-5">
LIMIT = 4096
</storycode:copy>
<a id="f-01359553f977-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-6">
count = 0
</storycode:copy>
<a id="f-01359553f977-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-7">
frames = {}
</storycode:copy>
<a id="f-01359553f977-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-8">
printing = threading.local()
</storycode:copy>
<a id="f-01359553f977-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-9">
next_call = 0
</storycode:copy>
<a id="f-01359553f977-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-10">
started = time.monotonic()

</storycode:copy>
<a id="f-01359553f977-location"></a>
### `location`

<storycode:copy id="f-01359553f977-location">
def location(frame):
    if frame is None: return None
    name = os.path.abspath(frame.f_code.co_filename)
    if name.startswith(ROOT + os.sep) and not name.endswith("observer.py"):
        return {"file": name, "line": frame.f_lineno - 1}
    return None

</storycode:copy>
<a id="f-01359553f977-task_id"></a>
### `task_id`

<storycode:copy id="f-01359553f977-task_id">
def task_id():
    try:
        task = asyncio.current_task()
        return str(id(task)) if task else None
    except RuntimeError: return None

</storycode:copy>
<a id="f-01359553f977-emit"></a>
### `emit`

<storycode:copy id="f-01359553f977-emit">
def emit(payload, frame=None, category=None, at_location=None):
    global count
    with LOCK:
        if OUT.closed or count >= LIMIT: return
        count += 1
        callers = []
        at = frame.f_back if frame else None
        while at:
            loc = location(at)
            if loc: callers.append(loc)
            at = at.f_back
        OUT.write(json.dumps({"payload": payload, "location": at_location or location(frame), "callers": callers,
            "invocation": frames.get(id(frame)), "parent_invocation": frames.get(id(frame.f_back)) if frame else None, "thread": str(threading.get_ident()), "task": task_id(),
            "time": time.monotonic() - started}) + "\n")
        OUT.flush()

</storycode:copy>
<a id="f-01359553f977-Buffer"></a>
### `Buffer`

<storycode:copy id="f-01359553f977-Buffer">
class Buffer:
    def __init__(self, name): self.name = name
    def write(self, data):
        data = bytes(data)
        emit({"kind": "output", "stream": self.name, "category": "buffer.write", "bytes": list(data)}, sys._getframe(1))
        return len(data)
    def flush(self): emit({"kind": "output", "stream": self.name, "category": "buffer.flush", "bytes": []}, sys._getframe(1))

</storycode:copy>
<a id="f-01359553f977-Stream"></a>
### `Stream`

<storycode:copy id="f-01359553f977-Stream">
class Stream:
    def __init__(self, name): self.name = name; self.buffer = Buffer(name)
    encoding = "utf-8"
    errors = "strict"
    def write(self, text):
        if not isinstance(text, str): raise TypeError("write expects str")
        emit({"kind": "output", "stream": self.name, "category": "print" if getattr(printing, "frame", None) else "write", "bytes": list(text.encode("utf-8"))}, getattr(printing, "frame", None) or sys._getframe(1))
        return len(text)
    def flush(self):
        if OUT.closed: return
        emit({"kind": "output", "stream": self.name, "category": "flush", "bytes": []}, getattr(printing, "frame", None) or sys._getframe(1))
    def isatty(self): return False
    def writable(self): return True
    def fileno(self): return 1 if self.name == "stdout" else 2

</storycode:copy>
<a id="f-01359553f977-trace"></a>
### `trace`

<storycode:copy id="f-01359553f977-trace">
def trace(frame, event, arg):
    global next_call
    if not location(frame): return None
    if event == "call":
        frame.f_trace_lines = False
        next_call += 1
        frames[id(frame)] = "call-" + str(next_call)
    elif event == "exception":
        typ, value, tb = arg
        if tb.tb_next is None:
            emit({"kind": "exception", "type_name": typ.__name__, "message": str(value), "handling": "throwing"}, frame)
    elif event == "return": frames.pop(id(frame), None)
    return trace

</storycode:copy>
<a id="f-01359553f977-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-11">
original_print = builtins.print
</storycode:copy>
<a id="f-01359553f977-observed_print"></a>
### `observed_print`

<storycode:copy id="f-01359553f977-observed_print">
def observed_print(*args, **kwargs):
    previous = getattr(printing, "frame", None)
    printing.frame = sys._getframe(1)
    try: return original_print(*args, **kwargs)
    finally: printing.frame = previous
</storycode:copy>
<a id="f-01359553f977-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-12">
builtins.print = observed_print
</storycode:copy>
<a id="f-01359553f977-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-13">
sys.stdout, sys.stderr = Stream("stdout"), Stream("stderr")
</storycode:copy>
<a id="f-01359553f977-expression_statement-14"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-14">
profiling = "--profile" in sys.argv[2:]
</storycode:copy>
<a id="f-01359553f977-expression_statement-15"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-15">
profiler = cProfile.Profile() if profiling else None
</storycode:copy>
<a id="f-01359553f977-if_statement"></a>
### `if_statement`

<storycode:copy id="f-01359553f977-if_statement">
if not profiling and "--control" not in sys.argv[2:]:
    sys.settrace(trace)
    threading.settrace(trace)
</storycode:copy>
<a id="f-01359553f977-expression_statement-16"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-16">
profile_started = time.monotonic() - started
</storycode:copy>
<a id="f-01359553f977-if_statement-2"></a>
### `if_statement`

<storycode:copy id="f-01359553f977-if_statement-2">
if profiler: profiler.enable()
</storycode:copy>
<a id="f-01359553f977-expression_statement-17"></a>
### `expression_statement`

<storycode:copy id="f-01359553f977-expression_statement-17">
status = "completed"
</storycode:copy>
<a id="f-01359553f977-try_statement"></a>
### `try_statement`

<storycode:copy id="f-01359553f977-try_statement">
try:
    runpy.run_path(sys.argv[1], run_name="__main__")
except BaseException as error:
    status = "failed"
    tb = error.__traceback__
    while tb and tb.tb_next: tb = tb.tb_next
    emit({"kind": "exception", "type_name": type(error).__name__, "message": str(error), "handling": "unhandled"}, tb.tb_frame if tb else None)
finally:
    if profiler:
        profiler.disable()
        profile_ended = time.monotonic() - started
        stats = pstats.Stats(profiler)
        for (file, line, name), (primitive, total, self_time, cumulative, callers) in stats.stats.items():
            function = {"name": name, "file": file, "line": max(0, line-1), "primitive_calls": primitive, "total_calls": total}
            for metric, value in (("self_seconds", self_time), ("cumulative_seconds", cumulative)):
                emit({"kind": "profile", "metric": metric, "value": value, "unit": "seconds",
                    "interval": {"start": {"domain": "provider-monotonic", "unit": "seconds", "value": profile_started},
                        "end": {"domain": "provider-monotonic", "unit": "seconds", "value": profile_ended}, "scope": "main-thread-profile"},
                    "aggregation": "function aggregate across calls; cumulative costs overlap and must not be summed", "samples": None, "function": function},
                    at_location={"file": os.path.abspath(file) if not file.startswith(("<", "~")) else file, "line": max(0, line-1)})
    sys.settrace(None)
    threading.settrace(None)
    with LOCK:
        OUT.write(json.dumps({"terminal": status, "truncated": count >= LIMIT, "elapsed": time.monotonic() - started,
            "python": sys.version, "provider_version": platform.python_version(), "environment": platform.platform(), "profiling": profiling, "coverage": ("Main-thread cProfile aggregates and output hooks; trace throwing/catching hooks disabled; native function mappings unresolved" if profiling else "Python text write/flush and trace throwing/unhandled; descriptor/native writes and catching sites unsupported; instrumentation is not tamper-resistant")}) + "\n")
        OUT.close()
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/python_observer.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/python_observer.py"><storycode:paste select="#f-01359553f977-expression_statement"/><storycode:paste select="#f-01359553f977-sys"/><storycode:paste select="#f-01359553f977-expression_statement-2"/><storycode:paste select="#f-01359553f977-expression_statement-3"/><storycode:paste select="#f-01359553f977-expression_statement-4"/><storycode:paste select="#f-01359553f977-expression_statement-5"/><storycode:paste select="#f-01359553f977-expression_statement-6"/><storycode:paste select="#f-01359553f977-expression_statement-7"/><storycode:paste select="#f-01359553f977-expression_statement-8"/><storycode:paste select="#f-01359553f977-expression_statement-9"/><storycode:paste select="#f-01359553f977-expression_statement-10"/><storycode:paste select="#f-01359553f977-location"/><storycode:paste select="#f-01359553f977-task_id"/><storycode:paste select="#f-01359553f977-emit"/><storycode:paste select="#f-01359553f977-Buffer"/><storycode:paste select="#f-01359553f977-Stream"/><storycode:paste select="#f-01359553f977-trace"/><storycode:paste select="#f-01359553f977-expression_statement-11"/><storycode:paste select="#f-01359553f977-observed_print"/><storycode:paste select="#f-01359553f977-expression_statement-12"/><storycode:paste select="#f-01359553f977-expression_statement-13"/><storycode:paste select="#f-01359553f977-expression_statement-14"/><storycode:paste select="#f-01359553f977-expression_statement-15"/><storycode:paste select="#f-01359553f977-if_statement"/><storycode:paste select="#f-01359553f977-expression_statement-16"/><storycode:paste select="#f-01359553f977-if_statement-2"/><storycode:paste select="#f-01359553f977-expression_statement-17"/><storycode:paste select="#f-01359553f977-try_statement"/></storycode:file>

</storycode:doc>
