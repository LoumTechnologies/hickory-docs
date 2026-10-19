"""Reviewed CPython hooks, run only inside the existing cell sandbox.
Not a security/attestation boundary against code modifying its interpreter.
"""
import sys, os, json, time, threading, asyncio, traceback, runpy, builtins
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

def emit(payload, frame=None, category=None):
    global count
    with LOCK:
        if count >= LIMIT: return
        count += 1
        callers = []
        at = frame.f_back if frame else None
        while at:
            loc = location(at)
            if loc: callers.append(loc)
            at = at.f_back
        OUT.write(json.dumps({"payload": payload, "location": location(frame), "callers": callers,
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
sys.settrace(trace)
threading.settrace(trace)
status = "completed"
try:
    runpy.run_path(sys.argv[1], run_name="__main__")
except BaseException as error:
    status = "failed"
    tb = error.__traceback__
    while tb and tb.tb_next: tb = tb.tb_next
    emit({"kind": "exception", "type_name": type(error).__name__, "message": str(error), "handling": "unhandled"}, tb.tb_frame if tb else None)
finally:
    sys.settrace(None)
    threading.settrace(None)
    with LOCK:
        OUT.write(json.dumps({"terminal": status, "truncated": count >= LIMIT, "elapsed": time.monotonic() - started,
            "python": sys.version, "coverage": "Python text write/flush and trace throwing/unhandled; descriptor/native writes and catching sites unsupported; instrumentation is not tamper-resistant"}) + "\n")
        OUT.close()
