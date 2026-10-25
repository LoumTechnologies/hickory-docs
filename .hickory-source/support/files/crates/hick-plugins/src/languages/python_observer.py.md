<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/python_observer.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `8f67e9fb9349403f4022571c2c13f9fd15ff30b6355241c2936d88af4271c216`.

<book:entry path="crates/hick-plugins/src/languages/python_observer.py">
"""Reviewed CPython hooks, run only inside the existing cell sandbox.
Not a security/attestation boundary against code modifying its interpreter.
"""
import sys, os, json, time, threading, asyncio, traceback, runpy, builtins, cProfile, pstats, platform
ROOT = os.getcwd()
OUT = open("observations.jsonl", "w", encoding="utf-8")
LOCK = threading.RLock()
LIMIT = 4096
count = 0
frames = {}
printing = threading.local()
next_call = 0
started = time.monotonic()

def location(frame):
    if frame is None: return None
    name = os.path.abspath(frame.f_code.co_filename)
    if name.startswith(ROOT + os.sep) and not name.endswith("observer.py"):
        return {"file": name, "line": frame.f_lineno - 1}
    return None

def task_id():
    try:
        task = asyncio.current_task()
        return str(id(task)) if task else None
    except RuntimeError: return None

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

class Buffer:
    def __init__(self, name): self.name = name
    def write(self, data):
        data = bytes(data)
        emit({"kind": "output", "stream": self.name, "category": "buffer.write", "bytes": list(data)}, sys._getframe(1))
        return len(data)
    def flush(self): emit({"kind": "output", "stream": self.name, "category": "buffer.flush", "bytes": []}, sys._getframe(1))

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

original_print = builtins.print
def observed_print(*args, **kwargs):
    previous = getattr(printing, "frame", None)
    printing.frame = sys._getframe(1)
    try: return original_print(*args, **kwargs)
    finally: printing.frame = previous
builtins.print = observed_print
sys.stdout, sys.stderr = Stream("stdout"), Stream("stderr")
profiling = "--profile" in sys.argv[2:]
profiler = cProfile.Profile() if profiling else None
if not profiling and "--control" not in sys.argv[2:]:
    sys.settrace(trace)
    threading.settrace(trace)
profile_started = time.monotonic() - started
if profiler: profiler.enable()
status = "completed"
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
</book:entry>

</book:doc>
