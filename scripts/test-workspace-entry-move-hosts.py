#!/usr/bin/env python3
"""Exercise a reviewed main-construction move against an owned real engine."""
import argparse
import importlib.util
import json
import os
import socket
import tempfile
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ORIGINAL = 'project.md'
DESTINATION = 'literal #main.spec'
PUBLISHED_SOURCE = ('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">'
                    '<h:project sources="part.md"/>'
                    '<h:file path="saved.txt"><h:paste select="#part"/></h:file>'
                    '</h:doc>\n')
DRAFT_SOURCE = PUBLISHED_SOURCE.replace('saved.txt', 'draft.txt')


def module(path, name):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def entry(origin):
    with urllib.request.urlopen(origin + '/api/workspace/entry', timeout=20) as reply:
        value = json.load(reply)
    assert value['version'] == 1 and value['source_effects'] is False, value
    return value['entry']


def browser_flow(driver, root, origin):
    first = entry(origin)
    assert first['path'] == ORIGINAL, first
    driver.act('open', path=ORIGINAL)
    driver.act('remember-editor', path=ORIGINAL)
    location = PUBLISHED_SOURCE.index('saved.txt')
    driver.act('source-text-select', path=ORIGINAL, anchor=location, head=location + len('saved.txt'))
    driver.act('type-source-selection', path=ORIGINAL, text='draft.txt')
    assert driver.act('construction-source', path=ORIGINAL) == DRAFT_SOURCE
    selection = driver.act('editor-position', path=ORIGINAL)
    size = driver.act('editor-size', path=ORIGINAL)

    def prepare():
        driver.act('change-open', command='Move main construction…')
        driver.act('change-prepare', name=DESTINATION, input='New workspace-relative path', command='Prepare move')
        return driver.act('change-resource-ready')

    def assert_source(path, absent, generation):
        stamp = entry(origin)
        assert stamp['identity'] == first['identity'] and stamp['generation'] == first['generation'] + generation, stamp
        assert stamp['path'] == path, stamp
        assert not (root / absent).exists()
        assert (root / path).read_text() == PUBLISHED_SOURCE
        assert driver.act('construction-source', path=path) == DRAFT_SOURCE
        assert driver.act('editor-is-retained', path=path, previous=ORIGINAL)
        assert driver.act('editor-position', path=path) == selection
        assert driver.act('editor-size', path=path) == size
        assert not any((root / path).exists() for path in ('saved.txt', 'draft.txt'))

    preview = prepare()
    assert DESTINATION in preview and 'saved.txt' in preview, preview
    assert_source(ORIGINAL, DESTINATION, 0)

    (root / DESTINATION).write_text('independent destination\n')
    refusal = driver.act('change-resource-save')
    assert 'Publish' in refusal or 'publication' in refusal.lower(), refusal
    assert (root / DESTINATION).read_text() == 'independent destination\n'
    assert (root / ORIGINAL).read_text() == PUBLISHED_SOURCE
    assert entry(origin) == first
    (root / DESTINATION).unlink()
    driver.act('change-close')

    prepare()
    result = driver.act('change-resource-save')
    assert 'Saved source resources' in result, result
    assert_source(DESTINATION, ORIGINAL, 1)
    for direction, path, absent, generation in [('Undo', ORIGINAL, DESTINATION, 2), ('Redo', DESTINATION, ORIGINAL, 3)]:
        driver.act('change-close')
        driver.act('change-open', command=direction + ' workspace change')
        driver.act('change-resource-ready')
        assert entry(origin)['generation'] == first['generation'] + generation - 1
        result = driver.act('change-resource-save')
        assert 'Saved source resources' in result, result
        assert_source(path, absent, generation)
    driver.act('change-close')
    driver.act('construction-open')
    reading = driver.act('construction-read', entry='')
    assert 'Captured entry: ' + DESTINATION in reading and 'saved.txt' in reading and 'draft.txt' in reading, reading
    connected = driver.act('behavior-ready')
    assert DESTINATION in connected, connected
    driver.act('construction-close')
    assert_source(DESTINATION, ORIGINAL, 3)
    driver.act('undo', path=DESTINATION)
    assert driver.act('construction-source', path=DESTINATION) == PUBLISHED_SOURCE
    driver.act('redo', path=DESTINATION)
    assert driver.act('construction-source', path=DESTINATION) == DRAFT_SOURCE
    print('PASS browser: main construction Move, stale destination refusal, reviewed Undo and Redo, unchanged saved baseline and retained editor Undo')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--host', choices=['browser', 'apple', 'android'], required=True)
    parser.add_argument('--engine', default=os.environ.get('HICKORY_SIMPLE_TEST_ENGINE', str(ROOT / 'target/source-edit-focus/debug/examples/engine_client')))
    args = parser.parse_args()
    fixture = module('scripts/test-simple-browser.py', 'simple_fixture')
    native = module('scripts/test-native-templates.py', 'native_fixture')
    with tempfile.TemporaryDirectory(prefix='entry-move-host-', dir=ROOT / 'target') as temporary:
        directory = Path(temporary)
        root = directory / 'workspace'
        root.mkdir()
        (root / '.git').mkdir()
        (root / '.hickory-entry-move-fixture').write_text('owned-entry-move-v1\n')
        (root / ORIGINAL).write_text(PUBLISHED_SOURCE)
        (root / 'part.md').write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:copy id="part">source</h:copy></h:doc>\n')
        with socket.socket() as reserve:
            reserve.bind(('127.0.0.1', 0))
            port = reserve.getsockname()[1]
        ui = f'http://127.0.0.1:{port}'
        env = {**os.environ, 'HICKORY_STATE_DIR': str(directory / 'state')}
        processes = []
        driver = None
        try:
            engine = fixture.start([args.engine, str(root), 'window', ui], env=env)
            processes.append(engine)
            origin = fixture.origin(engine)
            if args.host == 'browser':
                vite = fixture.start(['node', 'node_modules/vite/bin/vite.js', '--config', 'vite.simple.config.ts', '--host', '127.0.0.1', '--port', str(port), '--strictPort'], cwd=ROOT / 'apps/web', env={**env, 'HICKORY_API_ORIGIN': origin})
                processes.append(vite)
                driver = fixture.BrowserDriver(fixture.origin(vite) + '/simple.html')
                browser_flow(driver, root, origin)
            else:
                case = ('WorkspaceEntryMoveTests', 'testLiveMainMoveUndoRedo', 'entry-move') if args.host == 'apple' else ('WorkspaceEntryMoveTest', 'liveMainMoveUndoRedo', 'entry-move')
                native.run_host(args.host, origin, root, directory, fixture, acceptance=case)
            assert not (root / ORIGINAL).exists()
            assert (root / DESTINATION).read_text() == PUBLISHED_SOURCE
        finally:
            if driver:
                driver.close()
            for process in reversed(processes):
                fixture.stop(process)


if __name__ == '__main__':
    main()
