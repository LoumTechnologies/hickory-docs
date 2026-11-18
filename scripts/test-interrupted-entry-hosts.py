#!/usr/bin/env python3
"""Exercise reviewed interruption completion with a real engine and host."""
import argparse
from contextlib import ExitStack
import importlib.util
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import urllib.parse
import urllib.request


ROOT = Path(__file__).resolve().parents[1]


def load_script(name):
    path = ROOT / 'scripts' / name
    spec = importlib.util.spec_from_file_location(path.stem, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(origin, route):
    with urllib.request.urlopen(origin + '/api/' + route, timeout=20) as response:
        return json.load(response)


def authored_files(root):
    files = set()
    for path in root.rglob('*'):
        relative = path.relative_to(root)
        if path.is_file() and relative.parts[0] not in {'.hick-cache', '.git'}:
            files.add(relative.as_posix())
    return files


def replace_with_equal_bytes(root, path):
    destination = root / path
    inode = destination.stat().st_ino
    replacement = root / 'fixture-equal-byte-replacement.tmp'
    replacement.write_bytes(destination.read_bytes())
    replacement.replace(destination)
    assert destination.stat().st_ino != inode


def browser_completion(driver, root, origin, material, marker, cold, replaced_after_review):
    old = material['change']['before']['path']
    new = material['change']['after']['path']
    original = material['current_entry']
    saved = (root / material['effective_entry']).read_text()
    saved_scope = {path: (root / path).read_bytes() if (root / path).is_file() else None
                   for path in (old, new)}
    if cold:
        query = urllib.parse.urlencode({'id': material['request_id'], 'path': old, 'draft': 'before'})
        draft = read(origin, 'changes/resource-recovery?' + query)['draft_text']
        assert draft == marker['retained_text'] and draft != saved
    else:
        driver.act('open', path=old)
        driver.act('remember-editor', path=old)
        end = len(saved.encode('utf-16-le')) // 2
        driver.act('source-text-select', path=old, anchor=0, head=end)
        driver.act('type-source-selection', path=old, text=marker['live_text'])
        draft = driver.act('construction-source', path=old)
        assert draft == marker['live_text'] and draft != saved
        selection = driver.act('editor-position', path=old)
        geometry = driver.act('editor-size', path=old)
        captured = driver.act('source-text-state', path=old)

    def preview():
        driver.act('recovery-open')
        evidence = driver.act('recovery-interrupted')
        assert evidence['canComplete'] and old in evidence['text'] and new in evidence['text'], evidence
        notices = ' '.join(driver.act('recovery-prepare', completion=True))
        assert 'Review the saved source and retained draft' in notices, notices
        for path, expected in saved_scope.items():
            actual = (root / path).read_bytes() if (root / path).is_file() else None
            assert actual == expected, path
        assert read(origin, 'workspace/entry/interrupted')['interrupted']['material']['current_entry'] == original

    preview()
    if replaced_after_review:
        replace_with_equal_bytes(root, new)
        substituted = (root / new).stat().st_ino
        notice = ' '.join(driver.act('recovery-publish', completion=True))
        assert notice and 'Completed the reviewed Move' not in notice, notice
        current = read(origin, 'workspace/entry/interrupted')['interrupted']['material']
        assert current['current_entry'] == original and not current['can_complete'], current
        assert (root / new).stat().st_ino == substituted
        assert all((root / path).read_bytes() == expected for path, expected in saved_scope.items())
        assert driver.act('editor-is-retained', path=old, previous=old)
        assert driver.act('source-text-state', path=old) == captured
        assert driver.act('editor-size', path=old) == geometry
        assert not driver.act('recovery-interrupted')['canComplete']
        print('PASS browser refuses an equal-byte replacement after Preview and preserves both saved paths and the original draft')
        return
    if not cold:
        driver.act('construction-set-source', path=old, source=draft + '\n')
        driver.act('construction-history', redo=False)
        assert driver.act('construction-source', path=old) == draft
        refused = ' '.join(driver.act('recovery-publish', completion=True))
        assert 'Source changed after review' in refused, refused
        assert read(origin, 'workspace/entry/interrupted')['interrupted']['material']['current_entry'] == original
        for path, expected in saved_scope.items():
            actual = (root / path).read_bytes() if (root / path).is_file() else None
            assert actual == expected, path
        selection = driver.act('editor-position', path=old)
        captured = driver.act('source-text-state', path=old)
    driver.act('recovery-close')
    preview()
    notices = ' '.join(driver.act('recovery-publish', completion=True))
    assert 'Completed the reviewed Move' in notices, notices
    driver.act('recovery-close')
    assert read(origin, 'workspace/entry/interrupted')['interrupted'] is None
    stamp = read(origin, 'workspace/entry')['entry']
    assert stamp['identity'] == original['identity'] and stamp['path'] == new
    assert stamp['generation'] == material['reservation_floor'] + 1
    assert not (root / old).exists()
    assert (root / new).read_text() == saved
    if cold:
        driver.act('open', path=new)
    else:
        assert driver.act('editor-is-retained', path=new, previous=old)
        assert driver.act('editor-position', path=new) == selection
        assert driver.act('editor-size', path=new) == geometry
        assert driver.act('source-text-state', path=new) == captured
    assert driver.act('construction-source', path=new) == draft
    print('PASS browser recovery review and explicit completion of the sealed Move')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host', choices=['browser', 'apple', 'android'], required=True)
    parser.add_argument('--test-binary', type=Path, required=True, help='Focused hickory-cli test executable')
    parser.add_argument('--stage', choices=['copied', 'old_deleted'], default='copied')
    parser.add_argument('--cold', action='store_true', help='Check a browser with no original editor')
    parser.add_argument('--replaced', action='store_true', help='Check an equal-byte replacement of the copied destination')
    parser.add_argument('--replaced-after-review', action='store_true', help='Replace the copied destination after Preview')
    parser.add_argument('--engine', default=os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',
                        str(ROOT / 'target/source-edit-focus/debug/examples/engine_client')))
    args = parser.parse_args()
    if args.cold and args.host != 'browser':
        parser.error('Use --cold with the browser case')
    if args.stage == 'old_deleted' and not args.cold:
        parser.error('The old_deleted fixture starts without a retained original editor. Use --cold.')
    if args.replaced and (args.host != 'browser' or args.stage != 'copied'):
        parser.error('Use --replaced with the copied browser fixture')
    if args.replaced_after_review and (args.host != 'browser' or args.stage != 'copied' or args.replaced or args.cold):
        parser.error('Use --replaced-after-review with the live copied browser fixture')
    simple = load_script('test-simple-browser.py')
    owned = load_script('test-agent-proposal-persistence.py')
    native = load_script('test-native-templates.py')
    with tempfile.TemporaryDirectory(prefix='interrupted-entry-host-', dir=ROOT / 'target') as temporary:
        folder = Path(temporary)
        root, state = folder / 'workspace', folder / 'state'
        environment = {**os.environ, 'HICKORY_STATE_DIR': str(state), 'HICKORY_SIMPLE_TEST_ENGINE': args.engine}
        prepared = subprocess.run([sys.executable, str(ROOT / 'scripts/prepare-interrupted-entry.py'),
                        '--root', str(root), '--state', str(state), '--stage', args.stage,
                        '--test-binary', str(args.test_binary.resolve())], env=environment, cwd=ROOT,
                        capture_output=True, text=True, timeout=95)
        assert prepared.returncode == 0, (prepared.stdout, prepared.stderr)
        marker = json.loads((root / '.hick-cache/entry-completion-fixture.json').read_text())
        assert marker['kind'] == 'entry-interrupted-completion-fixture' and marker['version'] == 1
        assert Path(marker['source_root']).resolve() == root.resolve()
        initial_files = authored_files(root)
        expected_files = (initial_files - {marker['old_path']}) | {marker['destination']}
        if args.replaced:
            replace_with_equal_bytes(root, marker['destination'])
        with socket.socket() as reservation:
            reservation.bind(('127.0.0.1', 0))
            port = reservation.getsockname()[1]
        with ExitStack() as cleanup:
            engine = simple.start([args.engine, str(root), 'window', f'http://127.0.0.1:{port}'], env=environment)
            cleanup.callback(owned.stop_engine, simple, engine, state)
            origin = simple.origin(engine)
            material = read(origin, 'workspace/entry/interrupted')['interrupted']['material']
            assert material['can_complete'] is not args.replaced and material['source_files_modified'] is False, material
            assert material['request_id'] == marker['original_request_id']
            if args.host == 'browser':
                vite = simple.start(['node', 'node_modules/vite/bin/vite.js', '--config', 'vite.simple.config.ts',
                                     '--host', '127.0.0.1', '--port', str(port), '--strictPort'], cwd=ROOT / 'apps/web',
                                    env={**environment, 'HICKORY_API_ORIGIN': origin})
                cleanup.callback(simple.stop, vite)
                driver = simple.BrowserDriver(simple.origin(vite) + '/simple.html')
                cleanup.callback(driver.close)
                if args.replaced:
                    driver.act('recovery-open')
                    evidence = driver.act('recovery-interrupted')
                    assert not evidence['canComplete'] and material['notice'] in evidence['text'], evidence
                    assert read(origin, 'workspace/entry/interrupted')['interrupted']['material'] == material
                    assert authored_files(root) == initial_files
                    for path in (marker['old_path'], marker['destination']):
                        assert (root / path).read_text() == marker['saved_text']
                    print('PASS browser preserves an independently replaced destination and offers no completion')
                    return
                browser_completion(driver, root, origin, material, marker, args.cold, args.replaced_after_review)
                if args.replaced_after_review:
                    assert authored_files(root) == initial_files
                    return
            else:
                label = 'entry-interrupted'
                witness = (('InterruptedEntryTests', 'testReviewedEntryCompletion', label) if args.host == 'apple'
                           else ('InterruptedEntryTest', 'reviewedEntryCompletion', label))
                native.run_host(args.host, origin, root, folder, simple, acceptance=witness)
            assert read(origin, 'workspace/entry/interrupted')['interrupted'] is None
            assert authored_files(root) == expected_files, 'Recovery materialized files outside its authored source scope'


if __name__ == '__main__':
    main()
