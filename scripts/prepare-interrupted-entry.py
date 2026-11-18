#!/usr/bin/env python3
"""Prepare one owned interrupted namespace through the production publisher test."""
import argparse,importlib.util,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--root',required=True,type=Path);parser.add_argument('--state',required=True,type=Path);parser.add_argument('--test-binary',required=True,type=Path);parser.add_argument('--stage',choices=['copied','old_deleted'],required=True);args=parser.parse_args()
 root=args.root.resolve();state=args.state.resolve();binary=args.test_binary.resolve();assert binary.is_file() and os.access(binary,os.X_OK),'Use the current focused CLI libtest executable';assert not root.exists() and not state.exists(),'Fixture owns only new root/state directories';assert root.parent.is_dir() and state.parent.is_dir();root.mkdir(mode=0o700)
 move=module('entry_fixture_source',ROOT/'scripts/test-workspace-entry-transitions.py')
 for path,text in {'project.md':move.MAIN,'part.md':move.PART,'note.py':'readable = True\n'}.items():(root/path).write_bytes(text.encode())
 result=subprocess.run([str(binary),'serve::workspace_entry_authority::interrupted_fixture_tests::prepare_owned_interrupted_fixture','--exact','--ignored','--nocapture'],env={**os.environ,'HICKORY_STATE_DIR':str(state),'HICKORY_ENTRY_FIXTURE_ROOT':str(root),'HICKORY_ENTRY_FIXTURE_STAGE':args.stage},capture_output=True,text=True)
 if result.returncode:raise RuntimeError(result.stdout+result.stderr)
 lines=[line.partition('ENTRY_FIXTURE ')[2] for line in result.stdout.splitlines() if line.startswith('ENTRY_FIXTURE ')];assert len(lines)==1,'One exact production fixture case must report its sealed result';observed=json.loads(lines[0]);assert observed['stage']==args.stage and observed['original_result']['outcome']['complete'] is False
 assert '1 passed' in result.stdout and '0 failed' in result.stdout,'Ignored fixture must actually execute exactly once'
 marker={'version':1,'kind':'entry-interrupted-completion-fixture','stage':args.stage,'old_path':'project.md','destination':'next.md','saved_text':move.MAIN,'retained_text':move.DIRTY,'live_text':move.MAIN.replace('safe baseline','live retained completion body'),'original_request_id':'f'*32,'source_root':str(root),'state_dir':str(state),**observed}
 assert (root/'project.md').exists()==(args.stage=='copied');assert (root/'next.md').read_bytes()==move.MAIN.encode()
 (root/'.hick-cache/entry-completion-fixture.json').write_text(json.dumps(marker));print(json.dumps(marker),flush=True)
if __name__=='__main__':main()
