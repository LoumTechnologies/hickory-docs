#!/usr/bin/env python3
"""Execute exactly one named Vitest witness against integrated application source."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

BASE=Path(__file__).resolve().parents[1]
ROOT=BASE.parent/'apps/web'


def main():
    if len(sys.argv)!=3:raise ValueError('Expected a captured test file and one exact test title.')
    relative,name=sys.argv[1:]
    if not relative.startswith('apps/web/src/') or not relative.endswith(('.test.ts','.test.tsx')):
        raise ValueError('Expected an explicit frontend test file.')
    test=(BASE.parent/relative).resolve(strict=True)
    if not test.is_relative_to(ROOT.resolve()):raise ValueError('Test escapes integrated frontend.')
    dependencies=BASE.parent/'apps/web/node_modules'
    executable=dependencies/'vitest/vitest.mjs'
    if not executable.is_file():raise ValueError('Install the captured frontend dependencies before running this witness.')
    modules=ROOT/'node_modules'
    created=not os.path.lexists(modules)
    if created:modules.symlink_to(dependencies.resolve(),target_is_directory=True)
    try:
        with tempfile.TemporaryDirectory(prefix='hickory-focused-web-') as temporary:
            scratch=Path(temporary)
            config=scratch/'vite.config.mts'
            config.write_text('import base from '+json.dumps(str(ROOT/'vite.config.ts'))+';\n'
                +'export default {...base, cacheDir:'+json.dumps(str(scratch/'vite-cache'))
                +', test:{...base.test, cache:false, maxWorkers:1}};\n')
            report=scratch/'result.json'
            command=['node',str(executable),'run',str(test.relative_to(ROOT)),'--root',str(ROOT),
                '--config',str(config),'-t',re.escape(name)+'$','--reporter=json','--outputFile',str(report)]
            result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
            if result.returncode or not report.is_file():
                raise RuntimeError((result.stdout+'\n'+result.stderr)[-6000:])
            data=json.loads(report.read_text())
            passed=[a for suite in data.get('testResults',[]) for a in suite.get('assertionResults',[]) if a.get('status')=='passed']
            if not data.get('success') or data.get('numFailedTests')!=0 or len(passed)!=1 or passed[0].get('title')!=name:
                raise RuntimeError('Expected exactly the named passing assertion: '+json.dumps(data)[-4000:])
            print('WITNESS_RESULT '+json.dumps(dict(runner='vitest',passed=1,failed=0,file=relative,name=name,
                skipped=data.get('numPendingTests'),source_root=str(ROOT),dependency_root=str(dependencies.resolve()))))
    finally:
        if created:modules.unlink()


if __name__=='__main__':
    try:main()
    except (ValueError,RuntimeError) as error:
        print(str(error),file=sys.stderr);sys.exit(1)
