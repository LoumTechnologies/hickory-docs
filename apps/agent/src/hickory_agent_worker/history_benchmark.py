"""Replay curated historical C# changes with confined executable checks."""
import argparse
from dataclasses import asdict
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time
from .local import OllamaModel, LocalSettings

SDK='mcr.microsoft.com/dotnet/sdk:10.0'
RUNTIME='mcr.microsoft.com/dotnet/aspnet:10.0'
PROJECT='<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework><ImplicitUsings>disable</ImplicitUsings><Nullable>disable</Nullable><AssemblyName>Study</AssemblyName><EnableNETAnalyzers>false</EnableNETAnalyzers><UseSharedCompilation>false</UseSharedCompilation></PropertyGroup></Project>'
SYSTEM='You are completing a C# method in a small historical software task. Return a short cohesive literate document with your explanation and exactly one fenced csharp block containing the complete replacement method, including its signature. Supplied collaborators are read-only. Implement the task contract; do not change the test harness or access files directly, environment, reflection, network or process state; use only the supplied task collaborators for task data. A failing public check may be supplied for repair. Do not return JSON.'

def digest(value):
    return hashlib.sha256(value.encode()).hexdigest()

def load_dataset(path):
    data=json.loads(Path(path).read_text())
    if data.get('version')!=1 or data.get('cutoff_exclusive')!='2021-01-01':
        raise ValueError('Unknown historical dataset version or cutoff')
    ids=set()
    for case in data['cases']:
        if case['id'] in ids or not case['repository'].startswith('nloum/'):
            raise ValueError('Duplicate case or unexpected repository')
        ids.add(case['id'])
        if digest(case['answer'])!=case['answer_sha256']:
            raise ValueError('Historical answer hash mismatch')
        for key in ('author_date','committer_date'):
            if case['author'][key][:10]>='2021-01-01':
                raise ValueError('Case is outside the historical cutoff')
        if not case['public_tests'].strip() or not case['hidden_tests'].strip():
            raise ValueError('Both public and held-out checks are required')
    return data

def candidate_method(document):
    blocks=re.findall(r'^```(?:csharp|cs)\s*\n(.*?)^```\s*$',document,re.M|re.S)
    if len(blocks)!=1 or len(document.encode())>32768:
        raise ValueError('Return exactly one bounded csharp method block')
    if not blocks[0].strip():
        raise ValueError('Empty method block')
    return blocks[0].strip()+'\n'

def program(case,method,phase):
    if phase not in ('public','hidden'):
        raise ValueError('Unknown grading phase')
    tests=case[phase+'_tests']
    return case['usings']+('\npublic static class Subject {\n' if case.get('static_owner') else '\npublic class Subject {\n')+case['support']+'\n'+method+'\n'+case['assertions']+f'''\nstatic void Public() {{ {case['public_tests']} }}
static void Hidden() {{ {tests} }}
public static int Main() {{
 try {{ {('Public' if phase=='public' else 'Hidden')}(); Console.WriteLine("CHECKS_PASSED"); return 0; }}
 catch(Exception e) {{ Console.Error.WriteLine(e.GetType().Name+": "+e.Message); return 1; }}
}}
}}\n'''

def fixture_fingerprint(case):
    return digest(json.dumps([program(case,case['answer'],'public'),program(case,case['answer'],'hidden'),program(case,case['before'],'public')],sort_keys=True))

def validate_references(dataset,rows):
    if {r['case'] for r in rows}!={c['id'] for c in dataset['cases']} or len(rows)!=len(dataset['cases']):
        raise ValueError('Reference coverage differs from the dataset')
    for case in dataset['cases']:
        row=next(r for r in rows if r['case']==case['id'])
        if not row['eligible'] or row.get('fixture_sha256')!=fixture_fingerprint(case):
            raise ValueError('Reference witness is missing, failed, or stale')
        if not row['reference_public']['passed'] or not row['reference_hidden']['passed'] or row['before_public']['passed']:
            raise ValueError('Reference witness does not discriminate the change')

def invoke(command,timeout):
    started=time.monotonic()
    try:
        with tempfile.TemporaryFile() as output:
            result=subprocess.run(command,stdout=output,stderr=subprocess.STDOUT,timeout=timeout)
            output.seek(0);raw=output.read(65537)
        return dict(exit_code=result.returncode,output=raw[:65536].decode(errors='replace'),output_truncated=len(raw)>65536,elapsed_seconds=time.monotonic()-started,timeout=False)
    except subprocess.TimeoutExpired:
        return dict(exit_code=None,output='Evaluation exceeded its time budget',output_truncated=False,elapsed_seconds=time.monotonic()-started,timeout=True)

def grade(case,method,phase='public'):
    """Compile without network; execute only the resulting artifact in a second container."""
    with tempfile.TemporaryDirectory(prefix='hickory-history-') as directory:
        root=Path(directory);root.chmod(0o755)
        (root/'Program.cs').write_text(program(case,method,phase))
        (root/'Study.csproj').write_text(PROJECT)
        (root/'NuGet.Config').write_text('<configuration><packageSources><clear /></packageSources></configuration>')
        name='hickory-history-'+os.urandom(8).hex()
        common=['docker','run','--rm','--name',name,'--network','none','--cap-drop','ALL','--security-opt','no-new-privileges','--pids-limit','96','--cpus','2']
        try:
            build=invoke(common+['--memory','2g','--read-only','--tmpfs','/tmp:rw,size=384m','--user',f'{os.getuid()}:{os.getgid()}','-e','DOTNET_CLI_HOME=/tmp','-e','DOTNET_CLI_TELEMETRY_OPTOUT=1','-e','DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1','-v',f'{root}:/work:rw','-w','/work',SDK,'dotnet','build','Study.csproj','-o','out','--nologo','--verbosity','quiet'],90)
            if build['exit_code']!=0:
                return dict(passed=False,stage='compile',build=build,run=None)
            # The execution container receives binaries only, not driver/source files.
            execution=invoke(common+['--memory','512m','--read-only','--tmpfs','/tmp:rw,size=32m','--user','65534:65534','-v',f'{root / "out"}:/app:ro','-w','/app',RUNTIME,'dotnet','Study.dll'],20)
            return dict(passed=execution['exit_code']==0 and 'CHECKS_PASSED' in execution['output'],stage='execute',build=build,run=execution)
        finally:
            # A client timeout must not leave compiled candidate code running.
            subprocess.run(['docker','rm','-f',name],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=10)

def reading(case,current,feedback,layout='literate'):
    if layout not in ('literate','separate'):
        raise ValueError('Unknown historical reading layout')
    # Whitelist input fields: reference solutions, commit messages and hidden
    # test bodies never enter the model context.
    source='# Method to change\n\n```csharp\n'+current.rstrip()+'\n```'
    evidence='# Captured public check\n\n```text\n'+feedback.rstrip()+'\n```'
    support='# Read-only collaborators\n\n```csharp\n'+case['usings']+case['support']+'\n```'
    check='# Public acceptance example\n\n```csharp\n'+case['public_tests']+'\n```'
    blocks=[source,evidence,support,check] if layout=='literate' else [source,support,check,evidence]
    return '# Task\n\n'+case['prompt']+'\n\n'+'\n\n'.join(blocks)

def feedback_text(receipt):
    if receipt['passed']:
        return 'Public checks passed.'
    part=receipt['build'] if receipt['stage']=='compile' else receipt['run']
    return 'Public check failed during '+receipt['stage']+':\n'+part['output'][-6000:]

def image_identity():
    return {name:json.loads(subprocess.check_output(['docker','image','inspect',name,'--format','{{json .}}'],text=True))['Id'] for name in (SDK,RUNTIME)}

def verify_references(dataset,output):
    rows=[]
    for case in dataset['cases']:
        row=dict(case=case['id'],fixture_sha256=fixture_fingerprint(case),reference_public=grade(case,case['answer'],'public'),reference_hidden=grade(case,case['answer'],'hidden'),before_public=grade(case,case['before'],'public'))
        row['eligible']=row['reference_public']['passed'] and row['reference_hidden']['passed'] and not row['before_public']['passed']
        rows.append(row);Path(output).write_text(json.dumps(rows,indent=2)+'\n')
        print(json.dumps(dict(case=case['id'],eligible=row['eligible'],before_stage=row['before_public']['stage'])),flush=True)
    return rows

def open_model(model_name, settings, max_wait_seconds=30):
    """Wait only for the local transport's explicit memory-headroom refusal."""
    deadline = time.monotonic() + max_wait_seconds
    while True:
        try:
            return OllamaModel(model_name, settings)
        except RuntimeError as exc:
            if str(exc) != 'Insufficient available memory for this model plus context headroom':
                raise
            if time.monotonic() >= deadline:
                raise
            # Stopping Ollama can return before process memory is reclaimed.
            # Retrying construction never retries inference or weakens the guard.
            time.sleep(0.25)


def experiment(dataset,reference_rows,args):
    validate_references(dataset,reference_rows)
    settings=LocalSettings(context_tokens=8192,output_tokens=1536,temperature=0.2,response_format='text')
    output=args.output;output.mkdir(parents=True,exist_ok=True)
    config=dict(dataset_sha256=digest(json.dumps(dataset,sort_keys=True)),models=args.models,settings=asdict(settings),max_iterations=args.iterations,layout=args.layout,images=image_identity(),system_sha256=digest(SYSTEM))
    config_path=output/'configuration.json'
    if config_path.exists() and json.loads(config_path.read_text())!=config:
        raise ValueError('Use a new output directory for changed experiment conditions')
    config_path.write_text(json.dumps(config,indent=2)+'\n')
    all_rows=[]
    for model_name in args.models:
        model=open_model(model_name,settings)
        model_dir=output/model_name.replace(':','-');model_dir.mkdir(exist_ok=True)
        (model_dir/'model.json').write_text(json.dumps(model.identity,indent=2)+'\n')
        try:
            for case in dataset['cases']:
                directory=model_dir/case['id'];directory.mkdir(exist_ok=True)
                result_path=directory/'result.json'
                if result_path.exists():
                    all_rows.append(json.loads(result_path.read_text()));continue
                before=next(r for r in reference_rows if r['case']==case['id'])['before_public']
                current=case['before'];feedback=feedback_text(before);calls=[];candidate=None;accepted=False;started=time.monotonic()
                for turn in range(1,args.iterations+1):
                    messages=[dict(role='system',content=SYSTEM),dict(role='user',content=reading(case,current,feedback,args.layout))]
                    request=directory/f'turn-{turn}-request.json'
                    if request.exists():
                        request.rename(request.with_name(f'turn-{turn}-request-interrupted-{time.time_ns()}.json'))
                    request.write_text(json.dumps(messages,indent=2)+'\n')
                    record=dict(turn=turn)
                    try:
                        response=model.complete(messages)
                        (directory/f'turn-{turn}-response.md').write_text(response.text)
                        record=dict(turn=turn,finish_reason=response.finish_reason,metrics=response.metrics)
                        if response.finish_reason in ('length','max_tokens'):
                            raise ValueError('Output was truncated; candidate not executed')
                        candidate=candidate_method(response.text)
                        receipt=grade(case,candidate,'public')
                        record['public']=receipt;current=candidate;feedback=feedback_text(receipt)
                        accepted=receipt['passed']
                    except Exception as exc:
                        record['error']=str(exc);feedback='Request failed: '+str(exc)
                    calls.append(record)
                    (directory/f'turn-{turn}-receipt.json').write_text(json.dumps(record,indent=2)+'\n')
                    if accepted:break
                # Held-out feedback is never made available to another repair turn.
                hidden=grade(case,candidate,'hidden') if accepted else None
                row=dict(model=model_name,case=case['id'],kind=case['kind'],public_passed=accepted,hidden_passed=bool(hidden and hidden['passed']),iterations=len(calls),calls=calls,hidden=hidden,elapsed_seconds=time.monotonic()-started,candidate_sha256=digest(candidate) if candidate else None)
                if candidate:(directory/'candidate.cs').write_text(candidate)
                result_path.write_text(json.dumps(row,indent=2)+'\n');all_rows.append(row)
                (output/'report.json').write_text(json.dumps(all_rows,indent=2)+'\n')
                print(json.dumps({k:row[k] for k in ('model','case','public_passed','hidden_passed','iterations','elapsed_seconds')}),flush=True)
        finally:
            subprocess.run(['ollama','stop',model_name],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True)
    (output/'report.json').write_text(json.dumps(all_rows,indent=2)+'\n')
    return all_rows

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dataset',type=Path,required=True)
    parser.add_argument('--references',type=Path,required=True)
    parser.add_argument('--verify',action='store_true')
    parser.add_argument('--output',type=Path)
    parser.add_argument('--models',nargs='+',default=['qwen3-coder:30b','qwen3.8:27b'])
    parser.add_argument('--iterations',type=int,default=2)
    parser.add_argument('--layout',choices=['literate','separate'],default='literate')
    args=parser.parse_args(argv)
    if not 1<=args.iterations<=4:
        raise ValueError('Bound model repair iterations to 1..4')
    dataset=load_dataset(args.dataset)
    if args.verify:
        return 0 if all(r['eligible'] for r in verify_references(dataset,args.references)) else 1
    if not args.output:parser.error('--output is required for model trials')
    experiment(dataset,json.loads(args.references.read_text()),args)
    return 0

if __name__=='__main__':
    raise SystemExit(main())


