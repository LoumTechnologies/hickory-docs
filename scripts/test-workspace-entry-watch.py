#!/usr/bin/env python3
"""Check genuine folder and explicit authoring subscriptions in isolated fixtures."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[1]


def load(name, script):
    spec = importlib.util.spec_from_file_location(name, ROOT / script)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def entry(part):
    return f'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="{part}"/></h:doc>'


def content(output, text):
    return f'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="{output}">{text}</h:file></h:doc>'


def request(origin, route, body=None):
    data = None if body is None else json.dumps(body).encode()
    query = urllib.request.Request(origin + route, data=data, headers={'Content-Type': 'application/json'})
    try:
        with urllib.request.urlopen(query, timeout=20) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        detail = error.read(4096).decode('utf-8', errors='replace')
        raise AssertionError(f'{route} returned {error.code}: {detail}') from error


def publish_entry_transition(origin, proof, operation):
    intent = request(origin, '/api/workspace/entry/intent', {
        'version': 1, 'entry_selection': proof, 'operation': operation,
    })
    assert intent['before_entry'] == proof['entry'] and intent['source_files_modified'] is False, intent
    capture = {'context': intent['context'], 'edit': intent['edit']}
    inputs = request(origin, '/api/workspace/entry/inputs', {
        'version': 1, 'token': intent['token'], 'capture': capture,
    })
    assert inputs['source_files_modified'] is False and not inputs['refusals'], inputs
    proposal = {'inputs': inputs, 'capture': capture}
    if operation['kind'] == 'move':
        proposal['plan'] = request(origin, '/api/changes/plan-resources', {
            'context': intent['context'], 'title': intent['title'], 'edit': intent['edit'], 'sources': inputs['sources'],
        })['plan']
        consequences = request(origin, '/api/changes/resource-consequences', proposal)
        route = '/api/changes/resource-save'
        extra = {}
    else:
        proposal['recovery_id'] = operation['recovery_id']
        prepared = request(origin, '/api/changes/prepare-restoration', proposal)
        consequences = prepared['consequences']
        route = '/api/changes/restore-resources'
        extra = {'reviewed_recovery_fingerprint': operation['recovery_fingerprint']}
    identity = uuid.uuid4().hex
    receipt = request(origin, route, {'request_id': identity, 'reviewed_fingerprint': consequences['fingerprint'],
                                     'proposal': proposal, **extra})
    assert receipt['outcome']['complete'] and receipt['entry_outcome']['state'] == 'confirmed', receipt
    assert receipt['entry_outcome']['request_id'] == identity, receipt
    assert receipt['entry_outcome']['current_entry'] == intent['after_entry'], receipt
    return identity, intent['after_entry']


def move_retirement_suite(fixture, owned, binary, parent):
    root = parent / 'transition' / 'workspace'
    root.mkdir(parents=True)
    state = root.parent / 'state'
    env = {**os.environ, 'HICKORY_STATE_DIR': str(state)}
    main = root / 'project.md'
    new = root / 'literal #moved.spec'
    source = root / 'part.md'
    output = root / 'out.txt'
    moved_output = root / 'moved.txt'
    restored_output = root / 'restored.txt'
    main.write_text(entry('part.md'))
    source.write_text(content('out.txt', 'first'))
    engine = fixture.start([binary, str(root), 'window', 'http://127.0.0.1:1'], env=env)
    watchers = []
    try:
        origin = fixture.origin(engine)
        def watch(path):
            process = fixture.start([binary, '--watch', str(path)], env=env)
            watchers.append(process)
            return process
        def saved_output():
            return output.read_text() if output.is_file() else None
        def expect_output(process, path, text):
            def published():
                if process.poll() is not None:
                    raise RuntimeError(process.stdout.read())
                return path.read_text() if path.exists() else None
            try:
                fixture.eventually(published, text)
            except AssertionError as failure:
                fixture.stop(process)
                raise AssertionError(f'Watch did not publish {path.name}: {process.stdout.read()}') from failure
        def close_watch(process):
            process.send_signal(signal.SIGINT)
            assert process.wait(timeout=10) == 0, process.stdout.read()
        def original_publication_stays_retired(expected):
            deadline = time.monotonic() + 2.5
            while time.monotonic() < deadline:
                assert saved_output() == expected, (saved_output(), expected)
                time.sleep(.05)

        original_watch = watch(root)
        fixture.eventually(saved_output, 'first')
        original = request(origin, '/api/workspace/entry')
        recovery_id, moved = publish_entry_transition(origin, original, {'kind': 'move', 'destination': new.name})
        assert moved['path'] == new.name and moved['generation'] == original['entry']['generation'] + 1
        assert not main.exists() and new.read_text() == entry('part.md')
        source.write_text(content('out.txt', 'explicit moved address'))
        original_publication_stays_retired('first')

        unrecorded = subprocess.run([binary, '--watch', str(new)], env=env,
                                    capture_output=True, text=True, timeout=15)
        assert unrecorded.returncode != 0 and 'no recorded baseline for differing output' in unrecorded.stderr, unrecorded.stderr
        assert saved_output() == 'first'
        source.write_text(content(moved_output.name, 'explicit moved address'))
        original_publication_stays_retired('first')
        assert not moved_output.exists()
        explicit = watch(new)
        expect_output(explicit, moved_output, 'explicit moved address')
        assert saved_output() == 'first'
        close_watch(explicit)
        recovery = request(origin, '/api/changes/resource-recovery?id=' + urllib.parse.quote(recovery_id))
        assert recovery['entry_transition']['after_entry'] == moved, recovery
        proof = request(origin, '/api/workspace/entry')
        assert proof['entry'] == moved, proof
        _, restored = publish_entry_transition(origin, proof, {'kind': 'restoration', 'recovery_id': recovery_id,
                                                                  'recovery_fingerprint': recovery['fingerprint']})
        assert restored['path'] == main.name and restored['generation'] == moved['generation'] + 1, restored
        assert restored['identity'] == original['entry']['identity']
        assert not new.exists() and main.read_text() == entry('part.md')
        source.write_text(content(restored_output.name, 'fresh implicit role'))
        original_publication_stays_retired('first')
        assert not restored_output.exists()
        close_watch(original_watch)
        fresh = watch(root)
        expect_output(fresh, restored_output, 'fresh implicit role')
        assert moved_output.read_text() == 'explicit moved address'
    finally:
        for process in reversed(watchers):
            fixture.stop(process)
        owned.stop_engine(fixture, engine, state)
    print('PASS a moved and restored entry permanently retires the old watch; explicit moved and fresh implicit watches publish')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--move-retirement', action='store_true', help='Also exercise legacy watcher retirement through real Move and inverse')
    args = parser.parse_args()
    fixture = load("fixture", "scripts/test-simple-browser.py")
    owned = load("owned", "scripts/test-agent-proposal-persistence.py")
    binary = os.environ.get("HICKORY_SIMPLE_TEST_ENGINE", str(ROOT / "target/source-edit-focus/debug/examples/engine_client"))
    with tempfile.TemporaryDirectory(prefix="hickory-entry-watch-") as temporary:
        parent = Path(temporary).resolve()
        for explicit in (False, True):
            folder = parent / ("explicit" if explicit else "implicit")
            root = folder / "workspace"
            root.mkdir(parents=True)
            state = folder / "state"
            main = root / "project.md"
            selected = root / "literal #entry.spec" if explicit else main
            selected.write_text(entry("part.md"))
            if explicit:
                main.write_text(entry("other.md"))
                (root / "other.md").write_text(content("other.txt", "foreign"))
            source = root / "part.md"
            source.write_text(content("out.txt", "first"))
            output = root / "out.txt"
            watch = fixture.start(
                [binary, "--watch", str(selected if explicit else root)],
                env={**os.environ, "HICKORY_STATE_DIR": str(state)},
            )
            try:
                def read_output():
                    if watch.poll() is not None:
                        raise AssertionError(watch.stdout.read())
                    return output.read_text() if output.is_file() else None

                fixture.eventually(read_output, "first")
                source.write_text(content("out.txt", "second"))
                fixture.eventually(read_output, "second")
                assert selected.read_text() == entry("part.md")
                assert not (root / "other.txt").exists()
            finally:
                owned.stop_engine(fixture, watch, state)
            if explicit:
                plain = root / "plain.spec"
                plain.write_text("ordinary source")
                refused = subprocess.run(
                    [binary, "--watch", str(plain)],
                    env={**os.environ, "HICKORY_STATE_DIR": str(state)},
                    capture_output=True, text=True, timeout=15,
                )
                assert refused.returncode != 0 and "project" in refused.stderr.lower(), refused.stderr
                assert plain.read_text() == "ordinary source" and not (root / "other.txt").exists()
        if args.move_retirement:
            move_retirement_suite(fixture, owned, binary, parent)
    print("PASS implicit and explicit watch publication, repeated updates, exact explicit entry with no fallback")


if __name__ == "__main__":
    main()
