"""Measure document revision in disposable fixtures; no production edit authority."""
import argparse
import ast
from dataclasses import asdict
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile
import time
from .document_loop import DocumentAgent, DOCUMENT_INSTRUCTIONS
from .local import OllamaModel, LocalSettings
from .local_experiment import TASKS, evaluate
from .hosted import monty, LIMITS

class DocumentEvaluator:
    def __init__(self,task,session,hick,directory,polish=False):
        self.task,self.session,self.hick,self.directory=task,session,str(hick),directory
        self.document,self.code='', ''
        self.attempts=0
        self.polish,self.frozen_ast=polish,None

    def __call__(self,document,predecessor):
        if predecessor != hashlib.sha256(self.document.encode()).hexdigest():
            raise ValueError('Working document changed before evaluation')
        self.attempts+=1
        previous_document,previous_code=self.document,self.code
        self.document=document
        candidate=self.directory/f'candidate-{self.attempts}.md'
        candidate.write_text(document)
        receipt={'accepted':False,'scope':'isolated development candidate; no production review'}
        try:
            if len(document.encode()) > 20000:
                raise ValueError('Keep the complete candidate below 20 KiB')
            with tempfile.TemporaryDirectory(prefix='hickory-document-loop-') as scratch:
                root=Path(scratch)
                (root/'response.md').write_text(document)
                inventory=subprocess.run([self.hick,'project','generate','response.md','--root',str(root)],
                    capture_output=True,text=True,timeout=30)
                match=re.fullmatch(r'File ordinary (\d+) bytes solution.py\n?',inventory.stdout)
                if inventory.returncode or not match or int(match[1]) > 12000:
                    raise ValueError('Expected one bounded solution.py: '+(inventory.stdout+inventory.stderr)[-2000:])
                materialized=subprocess.run([self.hick,'project','materialize','response.md','--root',str(root)],
                    capture_output=True,text=True,timeout=30)
                if materialized.returncode:
                    raise ValueError(materialized.stderr[-2000:])
                candidate_code=(root/'solution.py').read_text()
                verified=subprocess.run([self.hick,'project','verify','response.md','--root',str(root)],
                    capture_output=True,text=True,timeout=30)
                if verified.returncode:
                    raise ValueError('Source-output verification failed: '+verified.stderr[-2000:])
                tree=ast.dump(ast.parse(candidate_code),include_attributes=False)
                if self.frozen_ast is not None and tree != self.frozen_ast:
                    self.document,self.code=previous_document,previous_code
                    receipt['retain_previous']=True
                    raise ValueError('Polishing changed executable structure. Preserve the previous algorithm and literals; edit prose, comments and formatting only.')
                if self.polish and self.frozen_ast is not None:
                    prose=re.sub(r'<([\w.-]+):(file|copy|entry|patch)\b[^>]*>.*?</\1:\2>', '', document, flags=re.S)
                    prose=re.sub(r'<[^>]+>', '', prose)
                    if len(re.findall(r'[A-Za-z]+',prose)) < 30 or not re.search(r'(?mi)^(?:#\s+|<h1(?:\s[^>]*)?>)',document):
                        raise ValueError('Add a title and at least 30 words of explanatory Markdown prose outside code. Explain the constraints and the chosen construction; code comments alone are not the literate explanation.')
                self.code=candidate_code
                checks=evaluate(self.session,self.code,self.task['public'])
                if self.polish and checks['passed'] and self.frozen_ast is None:
                    self.frozen_ast=tree
                receipt.update(accepted=checks['passed'],source_output_equal=True,public_checks=checks,
                               output_hash=hashlib.sha256(self.code.encode()).hexdigest(),
                               algorithm_guard='identical Python AST across polishing' if self.polish else None)
        except SyntaxError as exc:
            receipt['error']=(f'Generated solution.py is invalid Python at line {exc.lineno}: '
                + repr((exc.text or '').strip()) + '. File contents are literal Python; remove Markdown fences from inside hick:file. Keep explanatory Markdown outside the file tag.')
        except Exception as exc:
            receipt['error']=str(exc)
        (self.directory/f'candidate-{self.attempts}-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
        return receipt

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--model',default='qwen3-coder:30b')
    parser.add_argument('--hick',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--tasks',nargs='+',default=[t['id'] for t in TASKS])
    parser.add_argument('--max-iterations',type=int,default=6)
    parser.add_argument('--polish',action='store_true',help='Require a second, AST-preserving explanatory revision')
    args=parser.parse_args(argv)
    settings=LocalSettings(context_tokens=8192,output_tokens=1536,temperature=0.2,response_format='text')
    args.output.mkdir(parents=True,exist_ok=True)
    config=dict(model=args.model,settings=asdict(settings),tasks=args.tasks,max_iterations=args.max_iterations,
                polish=args.polish,instruction_sha256=hashlib.sha256(DOCUMENT_INSTRUCTIONS.encode()).hexdigest(),evaluator_version='ast-polish-v4-fresh-cases-heading',scope='Document-as-state feasibility pilot; not a controlled replacement for live agent UI')
    config_path=args.output/'configuration.json'
    if config_path.exists() and json.loads(config_path.read_text()) != config:
        raise ValueError('Use a new output directory for changed configuration')
    config_path.write_text(json.dumps(config,indent=2)+'\n')
    model=OllamaModel(args.model,settings)
    (args.output/'model.json').write_text(json.dumps(model.identity,indent=2)+'\n')
    results=[]
    for task in [t for t in TASKS if t['id'] in args.tasks]:
        directory=args.output/task['id']
        result_path=directory/'result.json'
        if result_path.exists():
            row=json.loads(result_path.read_text())
        else:
            if directory.exists(): directory.rename(directory.with_name(directory.name+'-interrupted-'+str(time.time_ns())))
            directory.mkdir(parents=True)
            def record(kind,data):
                with (directory/'journal.jsonl').open('a') as output:
                    output.write(json.dumps({'kind':kind,'data':data})+'\n')
            started=time.monotonic();start_calls=len(model.calls)
            with monty() as pool:
                with pool.checkout(limits=LIMITS) as session:
                    evaluator=DocumentEvaluator(task,session,args.hick.resolve(),directory,polish=args.polish)
                    agent=DocumentAgent(model,evaluator,record)
                    error,result=None,None
                    try:
                        result=agent.run(task['requirement']+'\nPublic samples:\n'+json.dumps(task['public']),max_iterations=args.max_iterations,polish=args.polish)
                    except Exception as exc:
                        error=str(exc)
                    hidden=evaluate(session,evaluator.code,task['hidden']) if evaluator.code else {'passed':False}
            calls=model.calls[start_calls:]
            row=dict(task=task['id'],completed=result is not None,error=error,hidden=hidden,
                     hidden_passed=hidden['passed'],iterations=evaluator.attempts,
                     elapsed_seconds=time.monotonic()-started,calls=calls,
                     input_tokens=sum(c.get('prompt_eval_count') or 0 for c in calls),
                     output_tokens=sum(c.get('eval_count') or 0 for c in calls))
            if result: (directory/'accepted.md').write_text(result.document)
            result_path.write_text(json.dumps(row,indent=2)+'\n')
        results.append(row)
        (args.output/'report.json').write_text(json.dumps(results,indent=2)+'\n')
        print(json.dumps({k:row[k] for k in ('task','completed','hidden_passed','iterations','elapsed_seconds','error')}),flush=True)
    return 0


if __name__=='__main__':
    raise SystemExit(main())


