<build:doc xmlns:build="http://www.hickorydocs.com/1.0">
# Compare local models with checkable historical methods

For: maintainers evaluating document-centered agent loops and context representations.

Start with the [benchmark guide](../../../../docs/developers/historical-code-benchmark.md) for the task contract and limitations. The chapter below owns the actual runner, safety witnesses and curated fixtures.

## dependencies

Use the installed local inference adapter and two known .NET container images. The response contract asks for readable prose and one C# method; internal JSON journals preserve exact requests without making JSON the model’s reading format.

<build:copy id="context-study-22263866-dependencies">
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

</build:copy>

## digest

Pin text used for reference integrity and trial configuration, so a stale solution or changed fixture cannot silently reuse old verification.

<build:copy id="context-study-22263866-digest">
def digest(value):
    return hashlib.sha256(value.encode()).hexdigest()

</build:copy>

## load dataset

Only this curated historical schema is accepted. Both author and committer dates must precede 2021, answers must match their retained hash, IDs must be unique, and every case must have public and held-out witnesses.

<build:copy id="context-study-22263866-load-dataset">
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

</build:copy>

## candidate method

A malformed or oversized document is a failed attempt, not executable fallback text. Extract one explicitly delimited C# block; this format check does not prove method semantics or resist adversarial test manipulation.

<build:copy id="context-study-22263866-candidate-method">
def candidate_method(document):
    blocks=re.findall(r'^```(?:csharp|cs)\s*\n(.*?)^```\s*$',document,re.M|re.S)
    if len(blocks)!=1 or len(document.encode())>32768:
        raise ValueError('Return exactly one bounded csharp method block')
    if not blocks[0].strip():
        raise ValueError('Empty method block')
    return blocks[0].strip()+'\n'

</build:copy>

## program

Wrap a historical method with explicit small collaborators and one selected check phase. A public evaluation must not compile held-out check bodies: compiler diagnostics are visible to the next repair attempt.

<build:copy id="context-study-22263866-program">
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

</build:copy>

## fixture fingerprint

Bind reference witnesses to all three exact compiled programs: passing public reference, passing held-out reference and failing broken/missing baseline. Metadata-only provenance corrections do not change this executable identity.

<build:copy id="context-study-22263866-fixture-fingerprint">
def fixture_fingerprint(case):
    return digest(json.dumps([program(case,case['answer'],'public'),program(case,case['answer'],'hidden'),program(case,case['before'],'public')],sort_keys=True))

</build:copy>

## validate references

Do not call the model when reference coverage is missing, duplicated, stale or non-discriminating. An eligible flag alone is insufficient; actual phase outcomes and fixture identity must agree.

<build:copy id="context-study-22263866-validate-references">
def validate_references(dataset,rows):
    if {r['case'] for r in rows}!={c['id'] for c in dataset['cases']} or len(rows)!=len(dataset['cases']):
        raise ValueError('Reference coverage differs from the dataset')
    for case in dataset['cases']:
        row=next(r for r in rows if r['case']==case['id'])
        if not row['eligible'] or row.get('fixture_sha256')!=fixture_fingerprint(case):
            raise ValueError('Reference witness is missing, failed, or stale')
        if not row['reference_public']['passed'] or not row['reference_hidden']['passed'] or row['before_public']['passed']:
            raise ValueError('Reference witness does not discriminate the change')

</build:copy>

## invoke

Bound compilation and execution time and retained output. A timeout is an observed failed check; the outer grader removes the named container even when the Docker client has timed out.

<build:copy id="context-study-22263866-invoke">
def invoke(command,timeout):
    started=time.monotonic()
    try:
        with tempfile.TemporaryFile() as output:
            result=subprocess.run(command,stdout=output,stderr=subprocess.STDOUT,timeout=timeout)
            output.seek(0);raw=output.read(65537)
        return dict(exit_code=result.returncode,output=raw[:65536].decode(errors='replace'),output_truncated=len(raw)>65536,elapsed_seconds=time.monotonic()-started,timeout=False)
    except subprocess.TimeoutExpired:
        return dict(exit_code=None,output='Evaluation exceeded its time budget',output_truncated=False,elapsed_seconds=time.monotonic()-started,timeout=True)

</build:copy>

## grade

Compile in a network-disabled SDK container, then execute only the produced binaries in a separate read-only, network-disabled runtime container. The runtime receives no source directory, credentials, Docker socket or workspace mount. This confines execution; finite check success is not a proof of correctness.

<build:copy id="context-study-22263866-grade">
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

</build:copy>

## reading

Whitelist the task, current method, read-only collaborators, public example and actual public feedback. Never serialize the full case into model context. Readable fenced code and text keep feedback beside its owner; the separate variant moves identical blocks without adding facts.

<build:copy id="context-study-22263866-reading">
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

</build:copy>

## feedback text

Only public evaluator output is available for repair. Limit diagnostics so one noisy compiler error cannot consume the entire next request.

<build:copy id="context-study-22263866-feedback-text">
def feedback_text(receipt):
    if receipt['passed']:
        return 'Public checks passed.'
    part=receipt['build'] if receipt['stage']=='compile' else receipt['run']
    return 'Public check failed during '+receipt['stage']+':\n'+part['output'][-6000:]

</build:copy>

## image identity

Record installed image identities with the inference configuration. No candidate chooses its own project, image, compiler arguments or mounted paths.

<build:copy id="context-study-22263866-image-identity">
def image_identity():
    return {name:json.loads(subprocess.check_output(['docker','image','inspect',name,'--format','{{json .}}'],text=True))['Id'] for name in (SDK,RUNTIME)}

</build:copy>

## verify references

Admit a historical method only when both check phases pass and its actual parent or generated missing-body baseline fails the public witness. Initial fixture mistakes remain excluded until corrected; they are not model failures.

<build:copy id="context-study-22263866-verify-references">
def verify_references(dataset,output):
    rows=[]
    for case in dataset['cases']:
        row=dict(case=case['id'],fixture_sha256=fixture_fingerprint(case),reference_public=grade(case,case['answer'],'public'),reference_hidden=grade(case,case['answer'],'hidden'),before_public=grade(case,case['before'],'public'))
        row['eligible']=row['reference_public']['passed'] and row['reference_hidden']['passed'] and not row['before_public']['passed']
        rows.append(row);Path(output).write_text(json.dumps(rows,indent=2)+'\n')
        print(json.dumps(dict(case=case['id'],eligible=row['eligible'],before_stage=row['before_public']['stage'])),flush=True)
    return rows

</build:copy>

## Wait for model memory to be reclaimed

The first live transition stopped before Qwen3.8 because Ollama acknowledged unloading Qwen3-Coder before its memory had been reclaimed. Retry only the transport's explicit headroom refusal for at most thirty seconds. Other errors remain immediate, the allocation guard remains authoritative, and this helper never sends an inference request. The original trial was resumed manually; its executed runner is retained separately from this orchestration fix.

<build:copy id="history-wait-for-model-headroom">
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


</build:copy>

## experiment

Each attempt sees the original task, current candidate and public feedback in a fresh request. Stop at public success and evaluate held-out checks once after repair stops. Retain failed and truncated attempts, refuse changed configurations on resume, and unload each model before testing the next.

<build:copy id="context-study-22263866-experiment">
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

</build:copy>

## main

Separate reference qualification from model trials. The CLI fixes the schema, bounds repair attempts and makes dataset, receipts and run destination explicit; it does not download models or publish artifacts.

<build:copy id="context-study-22263866-main">
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

</build:copy>

<build:file path="apps/agent/src/hickory_agent_worker/history_benchmark.py">
<build:paste select="#context-study-22263866-dependencies" /><build:paste select="#context-study-22263866-digest" /><build:paste select="#context-study-22263866-load-dataset" /><build:paste select="#context-study-22263866-candidate-method" /><build:paste select="#context-study-22263866-program" /><build:paste select="#context-study-22263866-fixture-fingerprint" /><build:paste select="#context-study-22263866-validate-references" /><build:paste select="#context-study-22263866-invoke" /><build:paste select="#context-study-22263866-grade" /><build:paste select="#context-study-22263866-reading" /><build:paste select="#context-study-22263866-feedback-text" /><build:paste select="#context-study-22263866-image-identity" /><build:paste select="#context-study-22263866-verify-references" /><build:paste select="#history-wait-for-model-headroom" /><build:paste select="#context-study-22263866-experiment" /><build:paste select="#context-study-22263866-main" />
</build:file>

## dependencies

Use isolated temporary artifacts and a fake model to witness information boundaries and cutoff refusal without invoking Ollama or Docker. Native reference qualification separately exercises actual C# compilation and execution.

<build:copy id="context-study-1128f21f-dependencies">
import copy
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
from hickory_agent_worker.core import ModelResponse
from hickory_agent_worker import history_benchmark as study

</build:copy>

## fixture

Give the safety witnesses distinct answer, commit-message and held-out sentinels so accidental disclosure can be detected.

<build:copy id="context-study-1128f21f-fixture">
def fixture():
    return dict(id='sample',repository='nloum/sample',kind='completion',author=dict(author_date='2020-12-31T23:00:00Z',committer_date='2020-12-31T23:00:00Z'),answer='private int Answer(){return 4;}',answer_sha256=study.digest('private int Answer(){return 4;}'),before='private int Answer(){throw new Exception();}',usings='using System;\n',support='// read-only collaborator',assertions='// assertions',public_tests='// PUBLIC_EXAMPLE',hidden_tests='// SECRET_HELD_OUT',prompt='Complete the method',commit_message='SECRET_COMMIT_MESSAGE')

</build:copy>

## HistoricalBenchmarkTests

Check oracle separation, identical layout information, stale reference refusal, historical cutoff and answer integrity, malformed document refusal and the requirement that truncated model text never reaches compilation.

<build:copy id="context-study-1128f21f-HistoricalBenchmarkTests">
class HistoricalBenchmarkTests(unittest.TestCase):
    def test_model_reading_has_same_information_without_oracles(self):
        case=fixture()
        a=study.reading(case,case['before'],'CURRENT_FAILURE','literate')
        b=study.reading(case,case['before'],'CURRENT_FAILURE','separate')
        self.assertNotEqual(a,b)
        self.assertCountEqual(a.splitlines(),b.splitlines())
        self.assertEqual(len(a.encode()),len(b.encode()))
        for text in (a,b):
            self.assertIn('CURRENT_FAILURE',text)
            self.assertIn('PUBLIC_EXAMPLE',text)
            self.assertNotIn('SECRET_HELD_OUT',text)
            self.assertNotIn('SECRET_COMMIT_MESSAGE',text)
            self.assertNotIn(case['answer'],text)

    def test_public_program_never_embeds_held_out_checks(self):
        case=fixture()
        self.assertNotIn('SECRET_HELD_OUT',study.program(case,case['before'],'public'))
        self.assertIn('SECRET_HELD_OUT',study.program(case,case['answer'],'hidden'))
        with self.assertRaises(ValueError):study.program(case,'','unknown')

    def test_references_are_bound_to_fixture_and_discriminate_missing_code(self):
        case=fixture();data=dict(cases=[case]);row=dict(case='sample',eligible=True,fixture_sha256=study.fixture_fingerprint(case),reference_public=dict(passed=True),reference_hidden=dict(passed=True),before_public=dict(passed=False))
        study.validate_references(data,[row])
        changed=copy.deepcopy(case);changed['hidden_tests']+=' changed'
        with self.assertRaises(ValueError):study.validate_references(dict(cases=[changed]),[row])
        row['before_public']['passed']=True
        with self.assertRaises(ValueError):study.validate_references(data,[row])

    def test_dates_and_answer_integrity_are_checked_before_inference(self):
        data=dict(version=1,cutoff_exclusive='2021-01-01',cases=[fixture()])
        with tempfile.TemporaryDirectory() as root:
            path=Path(root)/'dataset.json';path.write_text(json.dumps(data))
            study.load_dataset(path)
            data['cases'][0]['author']['committer_date']='2021-01-01T00:00:00Z';path.write_text(json.dumps(data))
            with self.assertRaises(ValueError):study.load_dataset(path)
            data['cases'][0]=fixture();data['cases'][0]['answer']+=' changed';path.write_text(json.dumps(data))
            with self.assertRaises(ValueError):study.load_dataset(path)

    def test_malformed_documents_are_refused(self):
        valid='Explanation\n```csharp\npublic int Value(){return 4;}\n```\n'
        self.assertIn('return 4',study.candidate_method(valid))
        for text in ('public int Value(){return 4;}',valid+valid,'```csharp\n\n```','x'*32769):
            with self.assertRaises(ValueError):study.candidate_method(text)

    def test_truncated_reply_is_never_compiled_or_executed(self):
        case=fixture();data=dict(cases=[case]);rows=[dict(case='sample',eligible=True,fixture_sha256=study.fixture_fingerprint(case),reference_public=dict(passed=True),reference_hidden=dict(passed=True),before_public=dict(passed=False,stage='execute',run=dict(output='NotImplementedException')))]
        fake=SimpleNamespace(identity={},complete=lambda messages:ModelResponse('```csharp\nprivate int Answer(){return 4;}\n```','length',{}))
        with tempfile.TemporaryDirectory() as root,patch.object(study,'OllamaModel',return_value=fake),patch.object(study,'image_identity',return_value={}),patch.object(study,'grade') as grade,patch.object(study.subprocess,'run'):
            result=study.experiment(data,rows,SimpleNamespace(output=Path(root),models=['fake'],iterations=1,layout='literate'))
            grade.assert_not_called()
            self.assertFalse(result[0]['public_passed'])
            self.assertFalse(result[0]['hidden_passed'])
            self.assertIn('truncated',result[0]['calls'][0]['error'])


    def test_memory_reclamation_wait_preserves_the_headroom_guard(self):
        ready=object()
        error=RuntimeError('Insufficient available memory for this model plus context headroom')
        with patch.object(study,'OllamaModel',side_effect=[error,ready]) as constructor,patch.object(study.time,'monotonic',side_effect=[0,1]),patch.object(study.time,'sleep') as sleep:
            self.assertIs(study.open_model('local',object()),ready)
            self.assertEqual(constructor.call_count,2)
            sleep.assert_called_once_with(0.25)
        with patch.object(study,'OllamaModel',side_effect=error),patch.object(study.time,'monotonic',side_effect=[0,31]),patch.object(study.time,'sleep') as sleep:
            with self.assertRaises(RuntimeError):study.open_model('local',object())
            sleep.assert_not_called()

    def test_unrelated_model_errors_are_not_retried(self):
        with patch.object(study,'OllamaModel',side_effect=RuntimeError('different transport failure')) as constructor,patch.object(study.time,'sleep') as sleep:
            with self.assertRaisesRegex(RuntimeError,'different transport'):
                study.open_model('local',object())
            constructor.assert_called_once()
            sleep.assert_not_called()


if __name__=='__main__':
    unittest.main()

</build:copy>

<build:file path="apps/agent/tests/test_history_benchmark.py">
<build:paste select="#context-study-1128f21f-dependencies" /><build:paste select="#context-study-1128f21f-fixture" /><build:paste select="#context-study-1128f21f-HistoricalBenchmarkTests" />
</build:file>


## Inspect the curated fixtures and their scope

The literal dataset is the maintained fixture source. Source blobs and the frozen curation script preserve where its methods came from; they are evidence, not scripts executed during reconstruction. The guide distinguishes qualified projections from complete original applications and human timing.

<build:entry path="docs/experiments/historical-code/2026-11-01/dataset.json">
{
  "version": 1,
  "cutoff_exclusive": "2021-01-01",
  "cases": [
    {
      "id": "codeio-exception-factory",
      "kind": "refactor",
      "repository": "nloum/CodeIO",
      "commit": "c850a1b023ead10e9cad72fe45984ead40d02801",
      "parent": "d46184b15ab068ef7e589707a5962333f1e40cb3",
      "path": "src/FluentSourceGenerators/Utilities.cs",
      "method": "MakeException",
      "source_line": 15,
      "normalized_source_sha256": "e0bdbf15baff8062f004da9704118f0a2d4e8643e16179d26b634dd9b1f81cbc",
      "source_blob_oid": "9f11f408e94acd1bd59be744dccb1a16011e7dd0",
      "source_raw_sha256": "e0bdbf15baff8062f004da9704118f0a2d4e8643e16179d26b634dd9b1f81cbc",
      "answer_sha256": "77d3286e50565e0539cf0db7954877923e33e93c07ac31b2a8a443233f0e6093",
      "author": {
        "author_name": "nloum",
        "author_email": "39776302+nloum@users.noreply.github.com",
        "author_date": "2020-12-22T07:24:39-06:00",
        "committer_date": "2020-12-22T07:24:39-06:00"
      },
      "prompt": "Refactor ThrowException into public static InvalidOperationException MakeException. It must return, rather than throw, the exception. Preserve optional CallerFilePath, CallerLineNumber and CallerMemberName attributes and append exactly \" (file:line in member)\" to the supplied message. The caller chooses whether to throw.",
      "prompt_origin": "Retrospective task contract curated from the commit and inspected behavior; not an original historical request",
      "before": "        public static void ThrowException(string errorMessage, [CallerFilePath] string callerFilePath = null,\n            [CallerLineNumber] int callerLineNumber = 0, [CallerMemberName] string callerMemberName = null)\n        {\n            throw new InvalidOperationException(errorMessage +\n                                                $\" ({callerFilePath}:{callerLineNumber} in {callerMemberName})\");\n        }",
      "answer": "        public static InvalidOperationException MakeException(string errorMessage, [CallerFilePath] string callerFilePath = null,\n            [CallerLineNumber] int callerLineNumber = 0, [CallerMemberName] string callerMemberName = null)\n        {\n            return new InvalidOperationException(errorMessage +\n                                                $\" ({callerFilePath}:{callerLineNumber} in {callerMemberName})\");\n        }",
      "usings": "using System;\nusing System.Collections.Generic;\nusing System.Collections.Immutable;\nusing System.Linq;\nusing System.IO;\nusing System.Text.RegularExpressions;\nusing System.Runtime.CompilerServices;\n",
      "support": "",
      "assertions": "\n    static void Equal<T>(T actual,T expected) { if(!object.Equals(actual,expected)) throw new Exception($\"Expected {expected}, got {actual}\"); }\n    static void True(bool value) { if(!value) throw new Exception(\"Assertion failed\"); }\n    static void Seq<T>(IEnumerable<T> actual,params T[] expected) { True(actual.SequenceEqual(expected)); }\n",
      "public_tests": "var e=MakeException(\"bad\", \"sample.cs\", 17, \"Build\"); Equal(e.Message,\"bad (sample.cs:17 in Build)\");",
      "hidden_tests": "var e=MakeException(\"\", \"x\", 0, \"\"); Equal(e.Message,\" (x:0 in )\"); var automatic=MakeException(\"automatic\"); True(automatic.Message.Contains(\"Program.cs:\")); True(automatic.Message.EndsWith(\" in Hidden)\"));",
      "static_owner": true,
      "projection": "Historical method with normalized newlines inside a reduced C# fixture; supplied collaborators are explicit test doubles. This is not a full original-project build.",
      "commit_url": "https://github.com/nloum/CodeIO/commit/c850a1b023ead10e9cad72fe45984ead40d02801",
      "source_url": "https://github.com/nloum/CodeIO/blob/c850a1b023ead10e9cad72fe45984ead40d02801/src/FluentSourceGenerators/Utilities.cs#L15",
      "group": "nloum/CodeIO@c850a1b023ead10e9cad72fe45984ead40d02801",
      "support_provenance": "Curated explicit fixture collaborators, not an original-project dependency build."
    },
    {
      "id": "codeio-strip-type-arguments",
      "kind": "completion",
      "repository": "nloum/CodeIO",
      "commit": "a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
      "parent": null,
      "path": "FluentSourceGenerators/Utilities.cs",
      "method": "GetWithoutTypeArguments",
      "source_line": 149,
      "normalized_source_sha256": "89c2f097f89b6df3c8ba7d51f7f3639aad882df944cdc3b760d295a3451ffcd6",
      "source_blob_oid": "6e178f3398fcb8a81673b29905cc4c1eb3453002",
      "source_raw_sha256": "311025d129bb7494926d076e98ddc182ba057f9a177e36d44a2af2437bf3f906",
      "answer_sha256": "5503375f17986cc9a75f82b55c1af7fc26e7924c50e9a217567fb752cf1a0b43",
      "author": {
        "author_name": "nloum",
        "author_email": "39776302+nloum@users.noreply.github.com",
        "author_date": "2020-10-31T07:38:59-05:00",
        "committer_date": "2020-10-31T07:38:59-05:00"
      },
      "prompt": "Implement GetWithoutTypeArguments: return the prefix before the first < character, or the original string if none. Keep whitespace unchanged. This helper does not parse or validate C# types.",
      "prompt_origin": "Retrospective task contract curated from the commit and inspected behavior; not an original historical request",
      "before": "        public static string GetWithoutTypeArguments(string type)\n        {\n            throw new NotImplementedException();\n        }",
      "answer": "        public static string GetWithoutTypeArguments(string type)\n        {\n            var idx = type.IndexOf('<');\n            if (idx >= 0)\n            {\n                return type.Substring(0, idx);\n            }\n\n            return type;\n        }",
      "usings": "using System;\nusing System.Collections.Generic;\nusing System.Collections.Immutable;\nusing System.Linq;\nusing System.IO;\nusing System.Text.RegularExpressions;\nusing System.Runtime.CompilerServices;\n",
      "support": "",
      "assertions": "\n    static void Equal<T>(T actual,T expected) { if(!object.Equals(actual,expected)) throw new Exception($\"Expected {expected}, got {actual}\"); }\n    static void True(bool value) { if(!value) throw new Exception(\"Assertion failed\"); }\n    static void Seq<T>(IEnumerable<T> actual,params T[] expected) { True(actual.SequenceEqual(expected)); }\n",
      "public_tests": "Equal(GetWithoutTypeArguments(\"Map<string, int>\"),\"Map\"); Equal(GetWithoutTypeArguments(\"Int32\"),\"Int32\");",
      "hidden_tests": "Equal(GetWithoutTypeArguments(\"\"),\"\"); Equal(GetWithoutTypeArguments(\"<T>\"),\"\"); Equal(GetWithoutTypeArguments(\" A <B<C>>\"),\" A \"); Equal(GetWithoutTypeArguments(\"A>B\"),\"A>B\");",
      "static_owner": true,
      "projection": "Historical method with normalized newlines inside a reduced C# fixture; supplied collaborators are explicit test doubles. This is not a full original-project build.",
      "commit_url": "https://github.com/nloum/CodeIO/commit/a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
      "source_url": "https://github.com/nloum/CodeIO/blob/a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631/FluentSourceGenerators/Utilities.cs#L149",
      "group": "nloum/CodeIO@a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
      "support_provenance": "Curated explicit fixture collaborators, not an original-project dependency build."
    },
    {
      "id": "codeio-variable-name",
      "kind": "completion",
      "repository": "nloum/CodeIO",
      "commit": "a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
      "parent": null,
      "path": "FluentSourceGenerators/Utilities.cs",
      "method": "GenerateVariableName",
      "source_line": 199,
      "normalized_source_sha256": "89c2f097f89b6df3c8ba7d51f7f3639aad882df944cdc3b760d295a3451ffcd6",
      "source_blob_oid": "6e178f3398fcb8a81673b29905cc4c1eb3453002",
      "source_raw_sha256": "311025d129bb7494926d076e98ddc182ba057f9a177e36d44a2af2437bf3f906",
      "answer_sha256": "1aa4360de31a34a6b2726b96a9860301ff2bb89060eb3ccf083b2e3e440eafa1",
      "author": {
        "author_name": "nloum",
        "author_email": "39776302+nloum@users.noreply.github.com",
        "author_date": "2020-10-31T07:38:59-05:00",
        "committer_date": "2020-10-31T07:38:59-05:00"
      },
      "prompt": "Implement GenerateVariableName. If isInterface is true remove the first character. Remove the first < and everything after it when present. Apply the supplied Camelize helper to the remaining text. Assume interface names are nonempty; preserve the behavior of the supplied helpers.",
      "prompt_origin": "Retrospective task contract curated from the commit and inspected behavior; not an original historical request",
      "before": "        public static string GenerateVariableName(string typeName, bool isInterface)\n        {\n            throw new NotImplementedException();\n        }",
      "answer": "        public static string GenerateVariableName(string typeName, bool isInterface)\n        {\n            var result = isInterface ? typeName.Substring(1) : typeName;\n            if (result.Contains(\"<\"))\n            {\n                result = result.Substring(0, result.IndexOf('<'));\n            }\n\n            return result.Camelize();\n        }",
      "usings": "using System;\nusing System.Collections.Generic;\nusing System.Collections.Immutable;\nusing System.Linq;\nusing System.IO;\nusing System.Text.RegularExpressions;\nusing System.Runtime.CompilerServices;\n",
      "support": "        public static string Pascalize(this string input)\n        {   \n            return Regex.Replace(input, \"(?:^|_| +)(.)\", match => match.Groups[1].Value.ToUpper());\n        }\n        public static string Camelize(this string input)\n        {\n            var word = input.Pascalize();\n            return word.Length > 0 ? word.Substring(0, 1).ToLower() + word.Substring(1) : word;\n        }",
      "assertions": "\n    static void Equal<T>(T actual,T expected) { if(!object.Equals(actual,expected)) throw new Exception($\"Expected {expected}, got {actual}\"); }\n    static void True(bool value) { if(!value) throw new Exception(\"Assertion failed\"); }\n    static void Seq<T>(IEnumerable<T> actual,params T[] expected) { True(actual.SequenceEqual(expected)); }\n",
      "public_tests": "Equal(GenerateVariableName(\"IReadOnlyList<T>\",true),\"readOnlyList\"); Equal(GenerateVariableName(\"DataStore\",false),\"dataStore\");",
      "hidden_tests": "Equal(GenerateVariableName(\"IFoo_Bar<T,U>\",true),\"fooBar\"); Equal(GenerateVariableName(\"\",false),\"\"); Equal(GenerateVariableName(\"I\",true),\"\"); Equal(GenerateVariableName(\"some name\",false),\"someName\");",
      "static_owner": true,
      "projection": "Historical method with normalized newlines inside a reduced C# fixture; supplied collaborators are explicit test doubles. This is not a full original-project build.",
      "commit_url": "https://github.com/nloum/CodeIO/commit/a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
      "source_url": "https://github.com/nloum/CodeIO/blob/a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631/FluentSourceGenerators/Utilities.cs#L199",
      "group": "nloum/CodeIO@a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
      "support_provenance": "Curated explicit fixture collaborators, not an original-project dependency build. Pascalize/Camelize are copied from CodeIO c850a1b, replacing the initial snapshot\u2019s Humanizer dependency; the target method itself is unchanged from its first introduction."
    },
    {
      "id": "codeio-cartesian-product",
      "kind": "completion",
      "repository": "nloum/CodeIO",
      "commit": "a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
      "parent": null,
      "path": "FluentSourceGenerators/Utilities.cs",
      "method": "CalcCombinationsOfOneFromEach",
      "source_line": 216,
      "normalized_source_sha256": "89c2f097f89b6df3c8ba7d51f7f3639aad882df944cdc3b760d295a3451ffcd6",
      "source_blob_oid": "6e178f3398fcb8a81673b29905cc4c1eb3453002",
      "source_raw_sha256": "311025d129bb7494926d076e98ddc182ba057f9a177e36d44a2af2437bf3f906",
      "answer_sha256": "05e13c3020b98ead751916570ffb404bee2914f84335cff7b5a505f5ec38f916",
      "author": {
        "author_name": "nloum",
        "author_email": "39776302+nloum@users.noreply.github.com",
        "author_date": "2020-10-31T07:38:59-05:00",
        "committer_date": "2020-10-31T07:38:59-05:00"
      },
      "prompt": "Implement a lazy Cartesian product over restartable finite sequences. Each result is an ImmutableList<TItem>. The first input dimension advances fastest. Empty outer input yields one empty combination. Any empty inner sequence throws ArgumentException when the result is enumerated. Do not change the signature.",
      "prompt_origin": "Retrospective task contract curated from the commit and inspected behavior; not an original historical request",
      "before": "        public static IEnumerable<ImmutableList<TItem>> CalcCombinationsOfOneFromEach<TItem>(\n            IEnumerable<IEnumerable<TItem>> enumerables ) {\n            throw new NotImplementedException();\n        }",
      "answer": "        public static IEnumerable<ImmutableList<TItem>> CalcCombinationsOfOneFromEach<TItem>(\n            IEnumerable<IEnumerable<TItem>> enumerables ) {\n            var enumerators = enumerables.Select( enumerable => Tuple.Create( enumerable, enumerable.GetEnumerator() ) ).ToList();\n\n            foreach ( var enumerator in enumerators ) {\n                if ( !enumerator.Item2.MoveNext() ) { throw new ArgumentException( \"An empty enumerable was specified\" );\n                }\n            }\n\n            yield return enumerators.Select( t => t.Item2.Current ).ToImmutableList();\n\n            while (true) {\n                var i = 0;\n                while(true) {\n                    if ( i == enumerators.Count ) {\n                        yield break;\n                    }\n\n                    if ( !enumerators[i].Item2.MoveNext() ) {\n                        enumerators[i] = Tuple.Create( enumerators[i].Item1,\n                            enumerators[i].Item1.GetEnumerator() );\n                        enumerators[i].Item2.MoveNext();\n                    } else {\n                        break;\n                    }\n\n                    i++;\n                }\n\n                yield return enumerators.Select( t => t.Item2.Current ).ToImmutableList();\n            }\n        }",
      "usings": "using System;\nusing System.Collections.Generic;\nusing System.Collections.Immutable;\nusing System.Linq;\nusing System.IO;\nusing System.Text.RegularExpressions;\nusing System.Runtime.CompilerServices;\n",
      "support": "",
      "assertions": "\n    static void Equal<T>(T actual,T expected) { if(!object.Equals(actual,expected)) throw new Exception($\"Expected {expected}, got {actual}\"); }\n    static void True(bool value) { if(!value) throw new Exception(\"Assertion failed\"); }\n    static void Seq<T>(IEnumerable<T> actual,params T[] expected) { True(actual.SequenceEqual(expected)); }\n",
      "public_tests": "var got=CalcCombinationsOfOneFromEach(new[]{new[]{1,2},new[]{3,4}}).Select(x=>string.Join(\",\",x)); Seq(got,\"1,3\",\"2,3\",\"1,4\",\"2,4\");",
      "hidden_tests": "var empty=CalcCombinationsOfOneFromEach(Array.Empty<IEnumerable<int>>()).ToList(); Equal(empty.Count,1); Equal(empty[0].Count,0); bool threw=false; try { CalcCombinationsOfOneFromEach(new[]{Array.Empty<int>()}).ToList(); } catch(ArgumentException) { threw=true; } True(threw); Seq(CalcCombinationsOfOneFromEach(new[]{new[]{\"a\",\"b\"},new[]{\"x\"},new[]{\"y\",\"z\"}}).Select(x=>string.Join(\"\",x)),\"axy\",\"bxy\",\"axz\",\"bxz\");",
      "static_owner": true,
      "projection": "Historical method with normalized newlines inside a reduced C# fixture; supplied collaborators are explicit test doubles. This is not a full original-project build.",
      "commit_url": "https://github.com/nloum/CodeIO/commit/a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
      "source_url": "https://github.com/nloum/CodeIO/blob/a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631/FluentSourceGenerators/Utilities.cs#L216",
      "group": "nloum/CodeIO@a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
      "support_provenance": "Curated explicit fixture collaborators, not an original-project dependency build."
    },
    {
      "id": "letters-next-image",
      "kind": "completion",
      "repository": "nloum/letter-classifier",
      "commit": "c97dd60d4a5061e5086f7c7e3db74ad6593d0afe",
      "parent": null,
      "path": "MainViewModel.cs",
      "method": "PrepareToClassifyNextImage",
      "source_line": 47,
      "normalized_source_sha256": "b91d3d5fb2987e986d1024b587d493a6caf683827b67e0ab369b8658f6a601c9",
      "source_blob_oid": "b57bb18cd7a02ab250cb82d63836f725f98247a9",
      "source_raw_sha256": "67a54442b05f8c8f726e7ca03b228def8e47b7a5d9762ee84bfa43ca45132d49",
      "answer_sha256": "2c768895ad94c0fe936a123b8f802dcd84261af28b676e66502274009abe4c3d",
      "author": {
        "author_name": "Nate Loum",
        "author_email": "nate.loum@trojans.dsu.edu",
        "author_date": "2019-05-12T21:59:39-05:00",
        "committer_date": "2019-05-12T21:59:39-05:00"
      },
      "prompt": "Implement PrepareToClassifyNextImage. Visit xValues in their existing order; choose the first X without a manual classification, set ImagePath.Value to _folderPath + \"\\\\image\" + X + \".png\", clear Letter.Value, and stop. If all values are classified (or the list is empty), leave both properties unchanged. Use the supplied small collaborators.",
      "prompt_origin": "Retrospective task contract curated from the commit and inspected behavior; not an original historical request",
      "before": "        private void PrepareToClassifyNextImage()\n        {\n            throw new NotImplementedException();\n        }",
      "answer": "        private void PrepareToClassifyNextImage()\n        {\n            foreach(var x in xValues)\n            {\n                if (!manualClassifications.Any(c => c.X == x))\n                {\n                    ImagePath.Value = _folderPath + \"\\\\image\" + x + \".png\";\n                    Letter.Value = \"\";\n                    break;\n                }\n            }\n        }",
      "usings": "using System;\nusing System.Collections.Generic;\nusing System.Collections.Immutable;\nusing System.Linq;\nusing System.IO;\nusing System.Text.RegularExpressions;\nusing System.Runtime.CompilerServices;\n",
      "support": "\n    public class Box<T> { public T Value; }\n    class ManualClassification { public int X; public string Letter; }\n    private string _folderPath=\"images\";\n    private List<ManualClassification> manualClassifications=new();\n    private List<int> xValues=new();\n    public Box<string> ImagePath=new();\n    public Box<string> Letter=new();\n    private static Regex _imageFileNameRegex=new Regex(@\"image(\\d+)\\.png\");\n",
      "assertions": "\n    static void Equal<T>(T actual,T expected) { if(!object.Equals(actual,expected)) throw new Exception($\"Expected {expected}, got {actual}\"); }\n    static void True(bool value) { if(!value) throw new Exception(\"Assertion failed\"); }\n    static void Seq<T>(IEnumerable<T> actual,params T[] expected) { True(actual.SequenceEqual(expected)); }\n",
      "public_tests": "var s=new Subject(); s.xValues.AddRange(new[]{9,2}); s.PrepareToClassifyNextImage(); Equal(s.ImagePath.Value,@\"images\\image9.png\"); Equal(s.Letter.Value,\"\");",
      "hidden_tests": "var s=new Subject(); s.xValues.AddRange(new[]{9,2,7}); s.manualClassifications.Add(new(){X=9}); s.PrepareToClassifyNextImage(); Equal(s.ImagePath.Value,@\"images\\image2.png\"); s.manualClassifications.AddRange(new[]{new ManualClassification(){X=2},new ManualClassification(){X=7}}); s.Letter.Value=\"keep\"; s.PrepareToClassifyNextImage(); Equal(s.Letter.Value,\"keep\"); var e=new Subject(); e.ImagePath.Value=\"old\"; e.Letter.Value=\"label\"; e.PrepareToClassifyNextImage(); Equal(e.ImagePath.Value,\"old\"); Equal(e.Letter.Value,\"label\");",
      "static_owner": false,
      "projection": "Historical method with normalized newlines inside a reduced C# fixture; supplied collaborators are explicit test doubles. This is not a full original-project build.",
      "commit_url": "https://github.com/nloum/letter-classifier/commit/c97dd60d4a5061e5086f7c7e3db74ad6593d0afe",
      "source_url": "https://github.com/nloum/letter-classifier/blob/c97dd60d4a5061e5086f7c7e3db74ad6593d0afe/MainViewModel.cs#L47",
      "group": "nloum/letter-classifier@c97dd60d4a5061e5086f7c7e3db74ad6593d0afe",
      "support_provenance": "Curated explicit fixture collaborators, not an original-project dependency build."
    },
    {
      "id": "letters-record-label",
      "kind": "completion",
      "repository": "nloum/letter-classifier",
      "commit": "c97dd60d4a5061e5086f7c7e3db74ad6593d0afe",
      "parent": null,
      "path": "MainViewModel.cs",
      "method": "LabelImage",
      "source_line": 92,
      "normalized_source_sha256": "b91d3d5fb2987e986d1024b587d493a6caf683827b67e0ab369b8658f6a601c9",
      "source_blob_oid": "b57bb18cd7a02ab250cb82d63836f725f98247a9",
      "source_raw_sha256": "67a54442b05f8c8f726e7ca03b228def8e47b7a5d9762ee84bfa43ca45132d49",
      "answer_sha256": "5911be703cdf288ad6cdd0c07e74f97ab0e19d2317f7234089f6867be00ef856",
      "author": {
        "author_name": "Nate Loum",
        "author_email": "nate.loum@trojans.dsu.edu",
        "author_date": "2019-05-12T21:59:39-05:00",
        "committer_date": "2019-05-12T21:59:39-05:00"
      },
      "prompt": "Implement LabelImage. Parse the integer captured by _imageFileNameRegex from the basename of ImagePath.Value. Append a ManualClassification with that X and the current Letter.Value, then call the supplied PrepareToClassifyNextImage. Assume a valid matching image path. Keep the label string exactly as entered.",
      "prompt_origin": "Retrospective task contract curated from the commit and inspected behavior; not an original historical request",
      "before": "        private void LabelImage()\n        {\n            throw new NotImplementedException();\n        }",
      "answer": "        private void LabelImage()\n        {\n            var match = _imageFileNameRegex.Match(Path.GetFileName(ImagePath.Value));\n            var capture = int.Parse(match.Groups[1].Value);\n\n            manualClassifications.Add(new ManualClassification\n            {\n                Letter = Letter.Value,\n                X = capture\n            });\n\n            PrepareToClassifyNextImage();\n        }",
      "usings": "using System;\nusing System.Collections.Generic;\nusing System.Collections.Immutable;\nusing System.Linq;\nusing System.IO;\nusing System.Text.RegularExpressions;\nusing System.Runtime.CompilerServices;\n",
      "support": "\n    public class Box<T> { public T Value; }\n    class ManualClassification { public int X; public string Letter; }\n    private string _folderPath=\"images\";\n    private List<ManualClassification> manualClassifications=new();\n    private List<int> xValues=new();\n    public Box<string> ImagePath=new();\n    public Box<string> Letter=new();\n    private static Regex _imageFileNameRegex=new Regex(@\"image(\\d+)\\.png\");\n\n        private void PrepareToClassifyNextImage()\n        {\n            foreach(var x in xValues)\n            {\n                if (!manualClassifications.Any(c => c.X == x))\n                {\n                    ImagePath.Value = _folderPath + \"\\\\image\" + x + \".png\";\n                    Letter.Value = \"\";\n                    break;\n                }\n            }\n        }",
      "assertions": "\n    static void Equal<T>(T actual,T expected) { if(!object.Equals(actual,expected)) throw new Exception($\"Expected {expected}, got {actual}\"); }\n    static void True(bool value) { if(!value) throw new Exception(\"Assertion failed\"); }\n    static void Seq<T>(IEnumerable<T> actual,params T[] expected) { True(actual.SequenceEqual(expected)); }\n",
      "public_tests": "var s=new Subject(); s.xValues.AddRange(new[]{7,4}); s.ImagePath.Value=@\"images\\image7.png\"; s.Letter.Value=\"Q\"; s.LabelImage(); Equal(s.manualClassifications.Count,1); Equal(s.manualClassifications[0].X,7); Equal(s.manualClassifications[0].Letter,\"Q\"); Equal(s.ImagePath.Value,@\"images\\image4.png\");",
      "hidden_tests": "var s=new Subject(); s.xValues.Add(123); s.ImagePath.Value=\"/images/image123.png\"; s.Letter.Value=\"a b\"; s.LabelImage(); Equal(s.manualClassifications.Single().X,123); Equal(s.manualClassifications.Single().Letter,\"a b\"); Equal(s.Letter.Value,\"a b\");",
      "static_owner": false,
      "projection": "Historical method with normalized newlines inside a reduced C# fixture; supplied collaborators are explicit test doubles. This is not a full original-project build.",
      "commit_url": "https://github.com/nloum/letter-classifier/commit/c97dd60d4a5061e5086f7c7e3db74ad6593d0afe",
      "source_url": "https://github.com/nloum/letter-classifier/blob/c97dd60d4a5061e5086f7c7e3db74ad6593d0afe/MainViewModel.cs#L92",
      "group": "nloum/letter-classifier@c97dd60d4a5061e5086f7c7e3db74ad6593d0afe",
      "support_provenance": "Curated explicit fixture collaborators, not an original-project dependency build."
    },
    {
      "id": "dependencies-to-project",
      "kind": "repair",
      "repository": "nloum/PackageProjectDependencySwitcher",
      "commit": "9fa4a1a917f491228f85c3b772ba8856bc5af7c0",
      "parent": "73ad92ac00f9dd8cdf62fc6704acc7e250741f5c",
      "path": "src/PackageProjectDependencySwitcher/DependencySwitcherService.cs",
      "method": "ConvertToProjectReferences",
      "source_line": 15,
      "normalized_source_sha256": "510be5ff65fd10eeae2fbd8ba85b564619dbe7988dae17b4fe741044839f2f4d",
      "source_blob_oid": "37b24f0fdf78f5850a7e40422d9b35033afcf879",
      "source_raw_sha256": "1c9193e9c2a2c5789fe1c22996f44d05e7e13eaecd23e383eaa8c66c291201e1",
      "answer_sha256": "cf2a5d4fd3b8c126fba41a29ebe421a3ade7778510772e8796b2c59193674529",
      "author": {
        "author_name": "nloum",
        "author_email": "39776302+nloum@users.noreply.github.com",
        "author_date": "2020-05-13T15:06:13-05:00",
        "committer_date": "2020-05-13T15:06:13-05:00"
      },
      "prompt": "Repair ConvertToProjectReferences so it converts PackageReference entries matching local versioned projects into ProjectReference entries relative to each consumer directory. Do not modify a project to reference itself; preserve unrelated entries and do not write unchanged files. Keep the public signature; referencePredicate and backup are unused in this historical contract. Inputs use simple self-closing XML entries and local projects have PackageVersion tags. Use supplied AbsolutePath collaborator.",
      "prompt_origin": "Retrospective task contract curated from the commit and inspected behavior; not an original historical request",
      "before": "        public void ConvertToProjectReferences(AbsolutePath solutionFolder, Func<AbsolutePath, string, bool> referencePredicate, bool backup)\n        {\n            var files = solutionFolder.GetDescendants(\"*.csproj\").ToImmutableList();\n\n            var filesToPackages = new Dictionary<AbsolutePath, Func<string, string>>();\n            \n            foreach (var file in files)\n            {\n                var text = file.ReadAllText();\n                var packageVersion = _packageVersion.Match(text);\n\n                if (packageVersion.Success)\n                {\n                    var regex = new Regex($\"<ProjectReference Include=\\\".+{file.Name}\\\" */>\");\n\n                    var packageVersionNumber = packageVersion.Groups[\"packageVersion\"].Value;\n                    filesToPackages[file] = str => regex.Replace(str, $\"<PackageReference Include=\\\"{file.WithoutExtension()}\\\" Version=\\\"{packageVersionNumber}\\\" />\");\n                }\n            }\n\n            foreach (var file in files)\n            {\n                var text = file.ReadAllText();\n\n                if (!referencePredicate(file, text))\n                {\n                    // TODO - make this behavior configurable.\n                    continue;\n                }\n                \n                var numChanges = 0;\n                foreach (var package in filesToPackages)\n                {\n                    var prevText = text;\n                    text = package.Value(text);\n                    if (!text.Equals(prevText))\n                    {\n                        numChanges++;\n                        Console.WriteLine($\"Changing {file.WithoutExtension()}'s dependency on {package.Key} from a project reference to a package reference\");\n                    }\n                }\n                \n                file.WriteAllText(text);\n            }\n        }",
      "answer": "        public void ConvertToProjectReferences(AbsolutePath solutionFolder, Func<AbsolutePath, string, bool> referencePredicate, bool backup)\n        {\n            var files = solutionFolder.GetDescendants(\"*.csproj\").ToImmutableList();\n\n            var filesToPackages = new Dictionary<AbsolutePath, Func<string, AbsolutePath, string>>();\n            \n            foreach (var file in files)\n            {\n                var text = file.ReadAllText();\n                var packageVersion = _packageVersion.Match(text);\n\n                if (packageVersion.Success)\n                {\n                    var regex = new Regex($\"<PackageReference Include=\\\"{Path.GetFileNameWithoutExtension(file.Name)}\\\".+/>\");\n\n                    filesToPackages[file] = (csprojFileText, relativeTo) =>\n                    {\n                        var path = file.RelativeTo(relativeTo);\n\n                        var result = regex.Replace(csprojFileText,\n                                $\"<ProjectReference Include=\\\"{path}\\\" />\");\n                        return result;\n                    };\n                }\n            }\n            \n            foreach (var file in files)\n            {\n                var text = file.ReadAllText();\n\n                var numChanges = 0;\n                foreach (var package in filesToPackages)\n                {\n                    if (package.Key.Equals(file))\n                    {\n                        continue;\n                    }\n                    \n                    var prevText = text;\n                    text = package.Value(text, file.Parent());\n                    if (!text.Equals(prevText))\n                    {\n                        numChanges++;\n                        Console.WriteLine($\"Changing {Path.GetFileNameWithoutExtension(file.ToString())}'s dependency on {Path.GetFileNameWithoutExtension(package.Key.ToString())} from a package reference to a project reference\");\n                    }\n                }\n\n                if (numChanges > 0)\n                {\n                    file.WriteAllText(text);\n                }\n            }\n        }",
      "usings": "using System;\nusing System.Collections.Generic;\nusing System.Collections.Immutable;\nusing System.Linq;\nusing System.IO;\nusing System.Text.RegularExpressions;\nusing System.Runtime.CompilerServices;\n",
      "support": "\n    public sealed class AbsolutePath {\n        public static readonly Dictionary<string,string> Files=new();\n        public static readonly List<string> Writes=new();\n        readonly string value;\n        public AbsolutePath(string value) { this.value=value; }\n        public string Name => Path.GetFileName(value);\n        public IEnumerable<AbsolutePath> GetDescendants(string pattern) => Files.Keys.Where(x=>x.StartsWith(value+\"/\") && x.EndsWith(\".csproj\")).Select(x=>new AbsolutePath(x)).ToList();\n        public string ReadAllText()=>Files[value];\n        public void WriteAllText(string text) { Files[value]=text; Writes.Add(value); }\n        public AbsolutePath Parent()=>new AbsolutePath(Path.GetDirectoryName(value));\n        public AbsolutePath WithoutExtension()=>new AbsolutePath(Path.ChangeExtension(value,null));\n        public bool HasExtension(string extension)=>value.EndsWith(extension);\n        public string RelativeTo(AbsolutePath parent)=>Path.GetRelativePath(parent.value,value);\n        public override string ToString()=>value;\n        public override bool Equals(object other)=>other is AbsolutePath p && p.value==value;\n        public override int GetHashCode()=>value.GetHashCode();\n    }\n    private static Regex _packageVersion = new Regex(@\"<PackageVersion>(?<packageVersion>[^<]+)</PackageVersion>\");\n    static void Setup(params (string,string)[] files) { AbsolutePath.Files.Clear(); AbsolutePath.Writes.Clear(); foreach(var (name,text) in files) AbsolutePath.Files[name]=text; }\n",
      "assertions": "\n    static void Equal<T>(T actual,T expected) { if(!object.Equals(actual,expected)) throw new Exception($\"Expected {expected}, got {actual}\"); }\n    static void True(bool value) { if(!value) throw new Exception(\"Assertion failed\"); }\n    static void Seq<T>(IEnumerable<T> actual,params T[] expected) { True(actual.SequenceEqual(expected)); }\n",
      "public_tests": "Setup((\"/s/Lib/Lib.csproj\",\"<Project><PackageVersion>1.2</PackageVersion></Project>\"),(\"/s/App/App.csproj\",\"<Project><PackageReference Include=\\\"Lib\\\" Version=\\\"0.1\\\" /></Project>\")); new Subject().ConvertToProjectReferences(new AbsolutePath(\"/s\"),(p,t)=>true,false); True(AbsolutePath.Files[\"/s/App/App.csproj\"].Contains(\"<ProjectReference Include=\\\"../Lib/Lib.csproj\\\" />\")); Equal(AbsolutePath.Writes.Count,1);",
      "hidden_tests": "Setup((\"/s/A/A.csproj\",\"<Project><PackageVersion>3</PackageVersion><PackageReference Include=\\\"A\\\" Version=\\\"1\\\" /></Project>\"),(\"/s/B/B.csproj\",\"<Project><PackageReference Include=\\\"External\\\" Version=\\\"9\\\" /></Project>\")); new Subject().ConvertToProjectReferences(new AbsolutePath(\"/s\"),(p,t)=>true,false); Equal(AbsolutePath.Writes.Count,0); True(AbsolutePath.Files[\"/s/B/B.csproj\"].Contains(\"External\"));",
      "static_owner": false,
      "projection": "Historical method with normalized newlines inside a reduced C# fixture; supplied collaborators are explicit test doubles. This is not a full original-project build.",
      "commit_url": "https://github.com/nloum/PackageProjectDependencySwitcher/commit/9fa4a1a917f491228f85c3b772ba8856bc5af7c0",
      "source_url": "https://github.com/nloum/PackageProjectDependencySwitcher/blob/9fa4a1a917f491228f85c3b772ba8856bc5af7c0/src/PackageProjectDependencySwitcher/DependencySwitcherService.cs#L15",
      "group": "nloum/PackageProjectDependencySwitcher@9fa4a1a917f491228f85c3b772ba8856bc5af7c0",
      "support_provenance": "Curated explicit fixture collaborators, not an original-project dependency build."
    },
    {
      "id": "dependencies-to-package",
      "kind": "repair",
      "repository": "nloum/PackageProjectDependencySwitcher",
      "commit": "9fa4a1a917f491228f85c3b772ba8856bc5af7c0",
      "parent": "73ad92ac00f9dd8cdf62fc6704acc7e250741f5c",
      "path": "src/PackageProjectDependencySwitcher/DependencySwitcherService.cs",
      "method": "ConvertToPackageReferences",
      "source_line": 69,
      "normalized_source_sha256": "510be5ff65fd10eeae2fbd8ba85b564619dbe7988dae17b4fe741044839f2f4d",
      "source_blob_oid": "37b24f0fdf78f5850a7e40422d9b35033afcf879",
      "source_raw_sha256": "1c9193e9c2a2c5789fe1c22996f44d05e7e13eaecd23e383eaa8c66c291201e1",
      "answer_sha256": "4fa125491f9b3c4229912b118f2c856fc4d6e11bb5f4fa37324e29d8c0858ff5",
      "author": {
        "author_name": "nloum",
        "author_email": "39776302+nloum@users.noreply.github.com",
        "author_date": "2020-05-13T15:06:13-05:00",
        "committer_date": "2020-05-13T15:06:13-05:00"
      },
      "prompt": "Repair ConvertToPackageReferences: replace ProjectReference entries targeting local versioned projects with PackageReference entries using the target filename without extension and its PackageVersion. Skip consumers whose project name without .csproj ends in .Test. Preserve unrelated content. Keep the signature and supplied AbsolutePath API. referencePredicate and backup are unused in this historical contract. Inputs use simple self-closing XML entries.",
      "prompt_origin": "Retrospective task contract curated from the commit and inspected behavior; not an original historical request",
      "before": "        public void ConvertToPackageReferences(AbsolutePath solutionFolder, Func<AbsolutePath, string, bool> referencePredicate, bool backup)\n        {\n            var files = solutionFolder.GetDescendants(\"*.csproj\").ToImmutableList();\n\n            var filesToPackages = new Dictionary<AbsolutePath, Func<string, AbsolutePath, string>>();\n            \n            foreach (var file in files)\n            {\n                var text = file.ReadAllText();\n                var packageVersion = _packageVersion.Match(text);\n\n                if (packageVersion.Success)\n                {\n                    var regex = new Regex($\"<PackageReference Include=\\\"{Path.GetFileNameWithoutExtension(file.ToString())}\\\"[^/]+/>\");\n\n                    filesToPackages[file] = (csprojFileText, relativeTo) =>\n                    {\n                        var path = file.RelativeTo(relativeTo);\n\n                        var result = regex.Replace(csprojFileText,\n                            $\"<PackageReference Include=\\\"{Path.GetFileNameWithoutExtension(file.ToString())}\\\" Version=\\\"{packageVersion.Groups[\"packageVersion\"].Value}\\\" />\");\n                        return result;\n                    };\n                }\n            }\n\n            foreach (var file in files)\n            {\n                var text = file.ReadAllText();\n\n                var numChanges = 0;\n                foreach (var package in filesToPackages)\n                {\n                    var prevText = text;\n                    text = package.Value(text, file.Parent());\n                    if (!text.Equals(prevText))\n                    {\n                        numChanges++;\n                        Console.WriteLine($\"Updating {Path.GetFileNameWithoutExtension(file.ToString())}'s dependency on {Path.GetFileNameWithoutExtension(package.Key.ToString())}\");\n                    }\n                }\n                \n                file.WriteAllText(text);\n            }\n        }",
      "answer": "        public void ConvertToPackageReferences(AbsolutePath solutionFolder, Func<AbsolutePath, string, bool> referencePredicate, bool backup)\n        {\n            var files = solutionFolder.GetDescendants(\"*.csproj\").ToImmutableList();\n\n            var filesToPackages = new Dictionary<AbsolutePath, Func<string, string>>();\n            \n            foreach (var file in files)\n            {\n                var text = file.ReadAllText();\n                var packageVersion = _packageVersion.Match(text);\n\n                if (packageVersion.Success)\n                {\n                    var regex = new Regex($\"<ProjectReference Include=\\\".+{file.Name}\\\" */>\");\n\n                    var packageVersionNumber = packageVersion.Groups[\"packageVersion\"].Value;\n                    filesToPackages[file] = str => regex.Replace(str, $\"<PackageReference Include=\\\"{file.WithoutExtension().Name}\\\" Version=\\\"{packageVersionNumber}\\\" />\");\n                }\n            }\n\n            foreach (var file in files)\n            {\n                var text = file.ReadAllText();\n\n                if (file.WithoutExtension().HasExtension(\".Test\"))\n                {\n                    // TODO - make this behavior configurable.\n                    continue;\n                }\n                \n                var numChanges = 0;\n                foreach (var package in filesToPackages)\n                {\n                    var prevText = text;\n                    text = package.Value(text);\n                    if (!text.Equals(prevText))\n                    {\n                        numChanges++;\n                        Console.WriteLine($\"Changing {file.WithoutExtension().Name}'s dependency on {package.Key.WithoutExtension().Name} from a project reference to a package reference\");\n                    }\n                }\n                \n                file.WriteAllText(text);\n            }\n        }",
      "usings": "using System;\nusing System.Collections.Generic;\nusing System.Collections.Immutable;\nusing System.Linq;\nusing System.IO;\nusing System.Text.RegularExpressions;\nusing System.Runtime.CompilerServices;\n",
      "support": "\n    public sealed class AbsolutePath {\n        public static readonly Dictionary<string,string> Files=new();\n        public static readonly List<string> Writes=new();\n        readonly string value;\n        public AbsolutePath(string value) { this.value=value; }\n        public string Name => Path.GetFileName(value);\n        public IEnumerable<AbsolutePath> GetDescendants(string pattern) => Files.Keys.Where(x=>x.StartsWith(value+\"/\") && x.EndsWith(\".csproj\")).Select(x=>new AbsolutePath(x)).ToList();\n        public string ReadAllText()=>Files[value];\n        public void WriteAllText(string text) { Files[value]=text; Writes.Add(value); }\n        public AbsolutePath Parent()=>new AbsolutePath(Path.GetDirectoryName(value));\n        public AbsolutePath WithoutExtension()=>new AbsolutePath(Path.ChangeExtension(value,null));\n        public bool HasExtension(string extension)=>value.EndsWith(extension);\n        public string RelativeTo(AbsolutePath parent)=>Path.GetRelativePath(parent.value,value);\n        public override string ToString()=>value;\n        public override bool Equals(object other)=>other is AbsolutePath p && p.value==value;\n        public override int GetHashCode()=>value.GetHashCode();\n    }\n    private static Regex _packageVersion = new Regex(@\"<PackageVersion>(?<packageVersion>[^<]+)</PackageVersion>\");\n    static void Setup(params (string,string)[] files) { AbsolutePath.Files.Clear(); AbsolutePath.Writes.Clear(); foreach(var (name,text) in files) AbsolutePath.Files[name]=text; }\n",
      "assertions": "\n    static void Equal<T>(T actual,T expected) { if(!object.Equals(actual,expected)) throw new Exception($\"Expected {expected}, got {actual}\"); }\n    static void True(bool value) { if(!value) throw new Exception(\"Assertion failed\"); }\n    static void Seq<T>(IEnumerable<T> actual,params T[] expected) { True(actual.SequenceEqual(expected)); }\n",
      "public_tests": "Setup((\"/s/Lib/Lib.csproj\",\"<Project><PackageVersion>1.2</PackageVersion></Project>\"),(\"/s/App/App.csproj\",\"<Project><ProjectReference Include=\\\"../Lib/Lib.csproj\\\" /></Project>\")); new Subject().ConvertToPackageReferences(new AbsolutePath(\"/s\"),(p,t)=>true,false); True(AbsolutePath.Files[\"/s/App/App.csproj\"].Contains(\"<PackageReference Include=\\\"Lib\\\" Version=\\\"1.2\\\" />\"));",
      "hidden_tests": "Setup((\"/s/Lib/Lib.csproj\",\"<Project><PackageVersion>7</PackageVersion></Project>\"),(\"/s/App.Test/App.Test.csproj\",\"<Project><ProjectReference Include=\\\"../Lib/Lib.csproj\\\" /></Project>\"),(\"/s/App/App.csproj\",\"<Project><ProjectReference Include=\\\"../Other/Other.csproj\\\" /></Project>\")); new Subject().ConvertToPackageReferences(new AbsolutePath(\"/s\"),(p,t)=>true,false); True(!AbsolutePath.Writes.Contains(\"/s/App.Test/App.Test.csproj\")); True(AbsolutePath.Files[\"/s/App/App.csproj\"].Contains(\"Other/Other.csproj\"));",
      "static_owner": false,
      "projection": "Historical method with normalized newlines inside a reduced C# fixture; supplied collaborators are explicit test doubles. This is not a full original-project build.",
      "commit_url": "https://github.com/nloum/PackageProjectDependencySwitcher/commit/9fa4a1a917f491228f85c3b772ba8856bc5af7c0",
      "source_url": "https://github.com/nloum/PackageProjectDependencySwitcher/blob/9fa4a1a917f491228f85c3b772ba8856bc5af7c0/src/PackageProjectDependencySwitcher/DependencySwitcherService.cs#L69",
      "group": "nloum/PackageProjectDependencySwitcher@9fa4a1a917f491228f85c3b772ba8856bc5af7c0",
      "support_provenance": "Curated explicit fixture collaborators, not an original-project dependency build."
    }
  ]
}
</build:entry>

<build:entry path="docs/experiments/historical-code/2026-11-01/provenance.json">
{
  "discovery": {
    "repos": [
      {
        "repository": "nloum/CSharpCream.NET",
        "url": "https://github.com/nloum/CSharpCream.NET",
        "size": 1490
      },
      {
        "repository": "nloum/CodeIO",
        "url": "https://github.com/nloum/CodeIO",
        "size": 1302
      },
      {
        "repository": "nloum/letter-classifier",
        "url": "https://github.com/nloum/letter-classifier",
        "size": 821
      },
      {
        "repository": "nloum/Replace-Package",
        "url": "https://github.com/nloum/Replace-Package",
        "size": 0
      },
      {
        "repository": "nloum/PackageProjectDependencySwitcher",
        "url": "https://github.com/nloum/PackageProjectDependencySwitcher",
        "size": 54
      },
      {
        "repository": "nloum/JoyOfCooking2018",
        "url": "https://github.com/nloum/JoyOfCooking2018",
        "size": 4839
      }
    ],
    "commits": [
      {
        "sha": "c850a1b023ead10e9cad72fe45984ead40d02801",
        "repository": "nloum/CodeIO",
        "author": {
          "login": "nloum",
          "name": null,
          "email": null,
          "avatar_url": "https://avatars.githubusercontent.com/u/39776302?v=4",
          "id": 39776302
        },
        "committer": {
          "login": "nathanloum",
          "name": null,
          "email": null,
          "avatar_url": "https://avatars.githubusercontent.com/u/131737109?v=4",
          "id": 131737109
        },
        "date": "2020-12-22T13:24:39Z",
        "url": "https://github.com/nloum/CodeIO/commit/c850a1b023ead10e9cad72fe45984ead40d02801",
        "message": "Don't throw exception in Utility method since that complicates control flow elsewhere"
      },
      {
        "sha": "a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
        "repository": "nloum/CodeIO",
        "author": {
          "login": "nloum",
          "name": null,
          "email": null,
          "avatar_url": "https://avatars.githubusercontent.com/u/39776302?v=4",
          "id": 39776302
        },
        "committer": {
          "login": "nathanloum",
          "name": null,
          "email": null,
          "avatar_url": "https://avatars.githubusercontent.com/u/131737109?v=4",
          "id": 131737109
        },
        "date": "2020-10-31T12:38:59Z",
        "url": "https://github.com/nloum/CodeIO/commit/a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
        "message": "Add basic structure of a fluent analyzer"
      },
      {
        "sha": "c97dd60d4a5061e5086f7c7e3db74ad6593d0afe",
        "repository": "nloum/letter-classifier",
        "author": {
          "login": "Nate Loum",
          "name": "Nate Loum",
          "email": null,
          "avatar_url": null,
          "id": null
        },
        "committer": {
          "login": "Nate Loum",
          "name": "Nate Loum",
          "email": null,
          "avatar_url": null,
          "id": null
        },
        "date": "2019-05-13T02:59:39Z",
        "url": "https://github.com/nloum/letter-classifier/commit/c97dd60d4a5061e5086f7c7e3db74ad6593d0afe",
        "message": "Write program for manually classifying letters"
      },
      {
        "sha": "9fa4a1a917f491228f85c3b772ba8856bc5af7c0",
        "repository": "nloum/PackageProjectDependencySwitcher",
        "author": {
          "login": "nloum",
          "name": null,
          "email": null,
          "avatar_url": "https://avatars.githubusercontent.com/u/39776302?v=4",
          "id": 39776302
        },
        "committer": {
          "login": "nloum",
          "name": null,
          "email": null,
          "avatar_url": "https://avatars.githubusercontent.com/u/39776302?v=4",
          "id": 39776302
        },
        "date": "2020-05-13T20:06:13Z",
        "url": "https://github.com/nloum/PackageProjectDependencySwitcher/commit/9fa4a1a917f491228f85c3b772ba8856bc5af7c0",
        "message": "Fix copy-paste error"
      }
    ]
  },
  "inventory": [
    {
      "repo": "nloum/CSharpCream.NET",
      "sha": "1061df466cba4a8d469e62cb322a739112b1da7f",
      "author_date": "2019-03-22T17:03:23-05:00",
      "committer_date": "2019-03-22T17:03:23-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Initial code drop",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CSharpCream.NET",
      "sha": "f3e12d0c7f5aa9f3340c1f835c1bc2ffd52b16c2",
      "author_date": "2019-03-22T17:00:00-05:00",
      "committer_date": "2019-03-22T17:00:00-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Initial commit",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "67a296c5381b18bfb72e757394240ed771ae207f",
      "author_date": "2020-12-22T07:37:58-06:00",
      "committer_date": "2020-12-22T07:37:58-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Add error message",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "c850a1b023ead10e9cad72fe45984ead40d02801",
      "author_date": "2020-12-22T07:24:39-06:00",
      "committer_date": "2020-12-22T07:24:39-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Don't throw exception in Utility method since that complicates control flow elsewhere",
      "has_selected_case": true
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "d46184b15ab068ef7e589707a5962333f1e40cb3",
      "author_date": "2020-12-22T07:20:45-06:00",
      "committer_date": "2020-12-22T07:20:45-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Add caller information to exception messages",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "3d3c71bb2d8a1ebbe622099509aabb792c06a447",
      "author_date": "2020-12-22T06:56:15-06:00",
      "committer_date": "2020-12-22T06:56:15-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Add more auto documentation",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "e1629bcc78a28920ba270e53c659e42cba306b77",
      "author_date": "2020-12-22T06:46:30-06:00",
      "committer_date": "2020-12-22T06:46:30-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Add documentation for generated extension methods",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "8e5d9aa01fe034571ed5f9545d4c2b0cc5046416",
      "author_date": "2020-12-22T06:38:46-06:00",
      "committer_date": "2020-12-22T06:38:46-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Start adding a generator for IoFluently",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "02b9984071853a1a6d7fd2e361ef39de58ad2fe9",
      "author_date": "2020-12-22T06:19:51-06:00",
      "committer_date": "2020-12-22T06:19:51-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Just push, don't test",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "ff35fdec21c2241d4c83ac5e623da6308597a095",
      "author_date": "2020-12-22T06:14:33-06:00",
      "committer_date": "2020-12-22T06:14:33-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Add better error messages",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "54af207b386abc3cf954d63226581428d0110b63",
      "author_date": "2020-12-21T13:54:43-06:00",
      "committer_date": "2020-12-21T13:54:43-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Add nullable reference type errors to help with stability",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "8f2b6e777a0060c80330506336dfbddcfaaa15ae",
      "author_date": "2020-12-21T12:41:21-06:00",
      "committer_date": "2020-12-21T12:41:21-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Fix namespaces",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "390c2d3288a06736879bfcb5fc84c20813dbe3b9",
      "author_date": "2020-12-21T12:40:41-06:00",
      "committer_date": "2020-12-21T12:40:41-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Updates after the .NET 5.0 release",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "6922799c128dc3acfcbad080591c3d87dd6b1350",
      "author_date": "2020-11-07T12:26:12-06:00",
      "committer_date": "2020-11-07T12:26:12-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Start documenting",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "6d143a8fa1195355665d1881a2ab953b55579a60",
      "author_date": "2020-11-07T12:25:54-06:00",
      "committer_date": "2020-11-07T12:25:54-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Remove Folder property since it's irrelevant now that we're a .NET source generator",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "c72e3cd77f1e9406c0792127c651c1c89f129ae7",
      "author_date": "2020-11-07T05:44:04-06:00",
      "committer_date": "2020-11-07T05:44:04-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Disable testing since there are no unit tests, and I'm testing with project references from LiveLinq and ComposableCollections for now",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "24b98d4bbf665c5ea881a499cbd36e0b78bf6e83",
      "author_date": "2020-11-07T05:42:24-06:00",
      "committer_date": "2020-11-07T05:42:24-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Fix bugs that caused LiveLinq build to fail",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "6f075d8534380f2518d10548100f13855a087a12",
      "author_date": "2020-11-06T10:38:47-06:00",
      "committer_date": "2020-11-06T10:38:47-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Try to get fluent source generator package working as analyzer",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "2ecf89d89617d3d3fc2119f522b8ae877c8bf2cd",
      "author_date": "2020-11-06T10:02:47-06:00",
      "committer_date": "2020-11-06T10:02:47-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Add CI",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "fe3b5689faa896f0dbc5c823e126972cc60f7e7b",
      "author_date": "2020-11-06T09:30:40-06:00",
      "committer_date": "2020-11-06T09:30:40-06:00",
      "author_name": "Nate Loum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Convert command line code generation tool to source generator",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "8f7663f54ab3317ae443804d0d217aa418d2a978",
      "author_date": "2020-11-06T09:30:40-06:00",
      "committer_date": "2020-11-06T09:30:40-06:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Convert command line code generation tool to source generator",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "dc63cb867a2f864064e65dbe8d2269fa5b7df179",
      "author_date": "2020-10-31T07:38:59-05:00",
      "committer_date": "2020-10-31T07:38:59-05:00",
      "author_name": "Nate Loum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Add basic structure of a fluent analyzer",
      "has_selected_case": false
    },
    {
      "repo": "nloum/CodeIO",
      "sha": "a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631",
      "author_date": "2020-10-31T07:38:59-05:00",
      "committer_date": "2020-10-31T07:38:59-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Add basic structure of a fluent analyzer",
      "has_selected_case": true
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "ca1105236b447996db3c296dc54a8777650ae8cc",
      "author_date": "2018-08-01T07:01:01-05:00",
      "committer_date": "2018-08-01T07:01:01-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Fix incorrect column name",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "76b5736f86647efa8b7e1e7269af7393b7ac9046",
      "author_date": "2018-08-01T06:23:39-05:00",
      "committer_date": "2018-08-01T06:32:07-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Fix recipe file names and remove duplicate LeftoverPotatoes",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "93e225895f339ee9cae2b7be30ad6026b9050ecb",
      "author_date": "2018-07-31T21:34:56-05:00",
      "committer_date": "2018-07-31T21:34:56-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "processed more RMD",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "db9b94e6bdf37ed2f637da33af97fd709054b67a",
      "author_date": "2018-07-31T17:37:45-04:00",
      "committer_date": "2018-07-31T17:37:45-04:00",
      "author_name": "joshwilkins2013",
      "author_email": "joshwilkins2013@gmail.com",
      "subject": "Cherry picked recipe additions into draft branch",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "59672c9f3a9b58cc0909a032d7dfb12292a5cc39",
      "author_date": "2018-07-31T17:33:45-04:00",
      "committer_date": "2018-07-31T17:33:45-04:00",
      "author_name": "joshwilkins2013",
      "author_email": "joshwilkins2013@gmail.com",
      "subject": "Cleaned up my preprocessing doc a little and added my master list to be analyzed",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "5036f50b0c821a290718d1da3ad008056b1b1746",
      "author_date": "2018-07-31T10:15:34-05:00",
      "committer_date": "2018-07-31T10:15:34-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "pushing some recipe table markdown source",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "8b146a7b0e052f66b792d3247aa3276c86a79afd",
      "author_date": "2018-07-31T10:12:19-05:00",
      "committer_date": "2018-07-31T10:12:19-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "correction for calories calculations in SimpleMultivariate",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "06c2f72cbc6eb1d07eead9c84650611d9a746712",
      "author_date": "2018-07-30T21:12:21-05:00",
      "committer_date": "2018-07-30T21:12:21-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "adding template files to markdown directory",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "4164d15c6be59e47fadad63571ffe9f9432acdf1",
      "author_date": "2018-07-30T21:11:02-05:00",
      "committer_date": "2018-07-30T21:11:02-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "two new recipes",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "35e2b7f127003ff85bf53c699b069adb9f12a3fb",
      "author_date": "2018-07-30T20:44:14-05:00",
      "committer_date": "2018-07-30T20:44:14-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "created a simple multivariate analysis document",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "d5c5dd73cfa89bb2b95e498ee23540635d76a4c8",
      "author_date": "2018-07-30T12:41:24-04:00",
      "committer_date": "2018-07-30T12:41:24-04:00",
      "author_name": "joshwilkins2013",
      "author_email": "joshwilkins2013@gmail.com",
      "subject": "Fixed file extensions, removed duplicates, made uniform file names. See JoshWilkins.Preprocessing.rmd in draft branch for more details if interested.",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "1b2f3f40fd2c99d64150adcdd0de8dd846dee197",
      "author_date": "2018-07-30T12:38:13-04:00",
      "committer_date": "2018-07-30T12:38:13-04:00",
      "author_name": "joshwilkins2013",
      "author_email": "joshwilkins2013@gmail.com",
      "subject": "Added my preprocessing rmd stuff for fixing the original recipe files",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "bf42bc05bdbaae29c87be0826ed45f736bb4d5a2",
      "author_date": "2018-07-29T11:51:38-04:00",
      "committer_date": "2018-07-29T11:51:38-04:00",
      "author_name": "joshwilkins2013",
      "author_email": "joshwilkins2013@gmail.com",
      "subject": "Added my recipe tab files",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "74c6faa19d413ada5c08a349ca59981d75cd2c8f",
      "author_date": "2018-07-27T13:56:00-05:00",
      "committer_date": "2018-07-27T13:56:00-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "38386167+PeterClaussenSDSU@users.noreply.github.com",
      "subject": "Create readme",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "36e85dc8f7f2f4221151e33fec106df54c2f81b8",
      "author_date": "2018-07-26T22:01:47-05:00",
      "committer_date": "2018-07-26T22:01:47-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "last set of tables for the draft version",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "7920f2022a820004e9101671a6274a853debd146",
      "author_date": "2018-07-26T18:42:04-05:00",
      "committer_date": "2018-07-26T18:42:04-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "More tables added",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "cdb89be4948dc27f3bc31ba108a8664709f593c4",
      "author_date": "2018-07-26T18:41:34-05:00",
      "committer_date": "2018-07-26T18:41:34-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "First 50 Rmd processed",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "dcbdd708c44e7fe1dcc5c284b690872d7aaa4522",
      "author_date": "2018-07-26T09:40:45-05:00",
      "committer_date": "2018-07-26T09:40:45-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "Updating recipe tables",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "9c031757ad1b9c27e448413a603fd8545baeeb02",
      "author_date": "2018-07-23T21:05:52-05:00",
      "committer_date": "2018-07-23T21:05:52-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "First release for class",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "12e8e3dc2ae66ffc4bea0573e67b922dcad782c2",
      "author_date": "2018-07-23T18:03:36-05:00",
      "committer_date": "2018-07-23T18:03:36-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "Peter.Claussen@sdstate.edu",
      "subject": "First set of recipe tables collated",
      "has_selected_case": false
    },
    {
      "repo": "nloum/JoyOfCooking2018",
      "sha": "3f523a5b1b7890f837db071341f6c15e1aeb2b01",
      "author_date": "2018-07-17T19:06:05-05:00",
      "committer_date": "2018-07-17T19:06:05-05:00",
      "author_name": "PeterClaussenSDSU",
      "author_email": "38386167+PeterClaussenSDSU@users.noreply.github.com",
      "subject": "Initial commit",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "3febde9926b7debf893f23a4bda03f51a9584442",
      "author_date": "2020-05-15T08:18:52-05:00",
      "committer_date": "2020-05-15T08:18:52-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Remove nuget version number so it doesn't override the GitVersion one",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "90fba8729b51b566daaa7cf2e00818c8eb156ee8",
      "author_date": "2020-05-13T15:46:41-05:00",
      "committer_date": "2020-05-13T15:46:41-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Try specifying patch",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "482049982b18dc277eccbf629d685d1d2c9a7e28",
      "author_date": "2020-05-13T15:42:59-05:00",
      "committer_date": "2020-05-13T15:42:59-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Clear artifacts directory before packing new nuget file",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "9fa4a1a917f491228f85c3b772ba8856bc5af7c0",
      "author_date": "2020-05-13T15:06:13-05:00",
      "committer_date": "2020-05-13T15:06:13-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Fix copy-paste error",
      "has_selected_case": true
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "73ad92ac00f9dd8cdf62fc6704acc7e250741f5c",
      "author_date": "2020-05-13T14:33:30-05:00",
      "committer_date": "2020-05-13T14:33:30-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Clarify API",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "9778eb53af2258cc852689ab762adc0f58f7d109",
      "author_date": "2020-05-13T14:25:22-05:00",
      "committer_date": "2020-05-13T14:25:22-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Add GitVersion package download",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "eebca9c112cf6b07949fd4e3c410796709af10a6",
      "author_date": "2020-05-13T14:17:11-05:00",
      "committer_date": "2020-05-13T14:17:11-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Setup gitversion and nuke.build",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "3d65c969c685e642f511de8c92a5050bc39f21e6",
      "author_date": "2020-05-12T16:46:53-05:00",
      "committer_date": "2020-05-12T16:46:53-05:00",
      "author_name": "Nate Loum",
      "author_email": "nathan.loum@bluecats.com",
      "subject": "Merge branch 'master' of https://github.com/nloum/PackageProjectDependencySwitcher",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "d93415cdcc6fbccbb8d53cee3e0c53bf5379eae5",
      "author_date": "2020-05-12T16:46:27-05:00",
      "committer_date": "2020-05-12T16:46:27-05:00",
      "author_name": "nloum",
      "author_email": "39776302+nloum@users.noreply.github.com",
      "subject": "Create dotnetcore.yml",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "c3672e6b66de1d5de279eb8b52f70f034064ff61",
      "author_date": "2020-05-12T16:44:52-05:00",
      "committer_date": "2020-05-12T16:44:52-05:00",
      "author_name": "Nate Loum",
      "author_email": "nathan.loum@bluecats.com",
      "subject": "Use template from utility disposables",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "e185c4c1622f74ab74cd72d27c46543fe655c12e",
      "author_date": "2020-05-12T16:41:51-05:00",
      "committer_date": "2020-05-12T16:41:51-05:00",
      "author_name": "Nate Loum",
      "author_email": "nathan.loum@bluecats.com",
      "subject": "nuke :setup:",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "0522336b1b56dfa934b760041e95f03b8dbdf79f",
      "author_date": "2020-05-12T16:40:58-05:00",
      "committer_date": "2020-05-12T16:40:58-05:00",
      "author_name": "Nate Loum",
      "author_email": "nathan.loum@bluecats.com",
      "subject": "Change to a library for use in Nuke.build",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "a6d4db6886bda0aadd56ce9219585cea72a2328a",
      "author_date": "2020-05-12T10:00:21-05:00",
      "committer_date": "2020-05-12T10:00:21-05:00",
      "author_name": "Nate Loum",
      "author_email": "nathan.loum@bluecats.com",
      "subject": "Update dependencies",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "1ac649b8b4bb0ed6cb94f51e2ee039fb1e58e151",
      "author_date": "2020-05-12T09:58:19-05:00",
      "committer_date": "2020-05-12T09:58:19-05:00",
      "author_name": "Nate Loum",
      "author_email": "nathan.loum@bluecats.com",
      "subject": "Fix bugs in updating references to packages",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "3c0dd004ac84cb96885f3f26dcb0946d12df37b6",
      "author_date": "2020-03-25T20:41:44-05:00",
      "committer_date": "2020-03-25T20:41:44-05:00",
      "author_name": "nloum",
      "author_email": "nathan.loum@bluecats.com",
      "subject": "Add readme and move classes into separate files for better code searchability",
      "has_selected_case": false
    },
    {
      "repo": "nloum/PackageProjectDependencySwitcher",
      "sha": "2815c0e1839bc854f1fc9db25e6367a316a7e434",
      "author_date": "2020-03-19T12:35:44-05:00",
      "committer_date": "2020-03-19T12:35:44-05:00",
      "author_name": "Nate Loum",
      "author_email": "nathan.loum@bluecats.com",
      "subject": "Initial code drop",
      "has_selected_case": false
    },
    {
      "repo": "nloum/letter-classifier",
      "sha": "740c552d8a62ea16c2be09008ee8b091412c0c5c",
      "author_date": "2019-05-14T13:44:09-05:00",
      "committer_date": "2019-05-14T13:44:09-05:00",
      "author_name": "Nate Loum",
      "author_email": "nate.loum@trojans.dsu.edu",
      "subject": "Label another 500 data points",
      "has_selected_case": false
    },
    {
      "repo": "nloum/letter-classifier",
      "sha": "990c722d4eafbcb5a864640d4f50696abbb551c5",
      "author_date": "2019-05-12T22:06:46-05:00",
      "committer_date": "2019-05-12T22:06:46-05:00",
      "author_name": "Nate Loum",
      "author_email": "nate.loum@trojans.dsu.edu",
      "subject": "Add executables so we can run it on any machine with the repo",
      "has_selected_case": false
    },
    {
      "repo": "nloum/letter-classifier",
      "sha": "c97dd60d4a5061e5086f7c7e3db74ad6593d0afe",
      "author_date": "2019-05-12T21:59:39-05:00",
      "committer_date": "2019-05-12T21:59:39-05:00",
      "author_name": "Nate Loum",
      "author_email": "nate.loum@trojans.dsu.edu",
      "subject": "Write program for manually classifying letters",
      "has_selected_case": true
    }
  ],
  "source_blobs": {
    "nloum/CodeIO@c850a1b023ead10e9cad72fe45984ead40d02801:src/FluentSourceGenerators/Utilities.cs": {
      "sha256": "e0bdbf15baff8062f004da9704118f0a2d4e8643e16179d26b634dd9b1f81cbc",
      "base64": "dXNpbmcgU3lzdGVtOwp1c2luZyBTeXN0ZW0uQ29sbGVjdGlvbnMuR2VuZXJpYzsKdXNpbmcgU3lzdGVtLkNvbGxlY3Rpb25zLkltbXV0YWJsZTsKdXNpbmcgU3lzdGVtLkxpbnE7CnVzaW5nIFN5c3RlbS5SdW50aW1lLkNvbXBpbGVyU2VydmljZXM7CnVzaW5nIFN5c3RlbS5UZXh0LlJlZ3VsYXJFeHByZXNzaW9uczsKdXNpbmcgTWljcm9zb2Z0LkNvZGVBbmFseXNpczsKdXNpbmcgTWljcm9zb2Z0LkNvZGVBbmFseXNpcy5DU2hhcnA7CnVzaW5nIE1pY3Jvc29mdC5Db2RlQW5hbHlzaXMuQ1NoYXJwLlN5bnRheDsKCm5hbWVzcGFjZSBGbHVlbnRTb3VyY2VHZW5lcmF0b3JzCnsKICAgIHB1YmxpYyBzdGF0aWMgY2xhc3MgVXRpbGl0aWVzCiAgICB7CiAgICAgICAgcHVibGljIHN0YXRpYyBJbnZhbGlkT3BlcmF0aW9uRXhjZXB0aW9uIE1ha2VFeGNlcHRpb24oc3RyaW5nIGVycm9yTWVzc2FnZSwgW0NhbGxlckZpbGVQYXRoXSBzdHJpbmcgY2FsbGVyRmlsZVBhdGggPSBudWxsLAogICAgICAgICAgICBbQ2FsbGVyTGluZU51bWJlcl0gaW50IGNhbGxlckxpbmVOdW1iZXIgPSAwLCBbQ2FsbGVyTWVtYmVyTmFtZV0gc3RyaW5nIGNhbGxlck1lbWJlck5hbWUgPSBudWxsKQogICAgICAgIHsKICAgICAgICAgICAgcmV0dXJuIG5ldyBJbnZhbGlkT3BlcmF0aW9uRXhjZXB0aW9uKGVycm9yTWVzc2FnZSArCiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICQiICh7Y2FsbGVyRmlsZVBhdGh9OntjYWxsZXJMaW5lTnVtYmVyfSBpbiB7Y2FsbGVyTWVtYmVyTmFtZX0pIik7CiAgICAgICAgfSAKICAgIAogICAgICAgIC8vLyA8c3VtbWFyeT4KICAgICAgICAvLy8gQnkgZGVmYXVsdCwgcGFzY2FsaXplIGNvbnZlcnRzIHN0cmluZ3MgdG8gVXBwZXJDYW1lbENhc2UgYWxzbyByZW1vdmluZyB1bmRlcnNjb3JlcwogICAgICAgIC8vLyA8L3N1bW1hcnk+CiAgICAgICAgLy8vIDxwYXJhbSBuYW1lPSJpbnB1dCI+PC9wYXJhbT4KICAgICAgICAvLy8gPHJldHVybnM+PC9yZXR1cm5zPgogICAgICAgIHB1YmxpYyBzdGF0aWMgc3RyaW5nIFBhc2NhbGl6ZSh0aGlzIHN0cmluZyBpbnB1dCkKICAgICAgICB7ICAgCiAgICAgICAgICAgIHJldHVybiBSZWdleC5SZXBsYWNlKGlucHV0LCAiKD86XnxffCArKSguKSIsIG1hdGNoID0+IG1hdGNoLkdyb3Vwc1sxXS5WYWx1ZS5Ub1VwcGVyKCkpOwogICAgICAgIH0KCiAgICAgICAgLy8vIDxzdW1tYXJ5PgogICAgICAgIC8vLyBTYW1lIGFzIFBhc2NhbGl6ZSBleGNlcHQgdGhhdCB0aGUgZmlyc3QgY2hhcmFjdGVyIGlzIGxvd2VyIGNhc2UKICAgICAgICAvLy8gPC9zdW1tYXJ5PgogICAgICAgIC8vLyA8cGFyYW0gbmFtZT0iaW5wdXQiPjwvcGFyYW0+CiAgICAgICAgLy8vIDxyZXR1cm5zPjwvcmV0dXJucz4KICAgICAgICBwdWJsaWMgc3RhdGljIHN0cmluZyBDYW1lbGl6ZSh0aGlzIHN0cmluZyBpbnB1dCkKICAgICAgICB7CiAgICAgICAgICAgIHZhciB3b3JkID0gaW5wdXQuUGFzY2FsaXplKCk7CiAgICAgICAgICAgIHJldHVybiB3b3JkLkxlbmd0aCA+IDAgPyB3b3JkLlN1YnN0cmluZygwLCAxKS5Ub0xvd2VyKCkgKyB3b3JkLlN1YnN0cmluZygxKSA6IHdvcmQ7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHB1YmxpYyBzdGF0aWMgU3ludGF4Tm9kZT8gRmluZFBhcmVudE9mS2luZChTeW50YXhOb2RlIGNoaWxkLCBTeW50YXhLaW5kIHN5bnRheEtpbmQpCiAgICAgICAgewogICAgICAgICAgICBpZiAoY2hpbGQuSXNLaW5kKHN5bnRheEtpbmQpKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICByZXR1cm4gY2hpbGQ7CiAgICAgICAgICAgIH0KCiAgICAgICAgICAgIGlmIChjaGlsZC5QYXJlbnQgIT0gbnVsbCkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgcmV0dXJuIEZpbmRQYXJlbnRPZktpbmQoY2hpbGQuUGFyZW50LCBzeW50YXhLaW5kKTsKICAgICAgICAgICAgfQoKICAgICAgICAgICAgcmV0dXJuIG51bGw7CiAgICAgICAgfQoKICAgICAgICBwdWJsaWMgc3RhdGljIEltbXV0YWJsZUxpc3Q8VD4gR2V0RGVzY2VuZGFudHNPZlR5cGU8VD4oU3ludGF4Tm9kZSBub2RlKSB3aGVyZSBUIDogU3ludGF4Tm9kZQogICAgICAgIHsKICAgICAgICAgICAgdmFyIHJlc3VsdCA9IG5ldyBMaXN0PFQ+KCk7CiAgICAgICAgICAgIFRyYXZlcnNlVHJlZShub2RlLCBkZXNjZW5kYW50ID0+CiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIGlmIChkZXNjZW5kYW50IGlzIFQgdCkKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICByZXN1bHQuQWRkKHQpOwogICAgICAgICAgICAgICAgfQogICAgICAgICAgICB9KTsKICAgICAgICAgICAgcmV0dXJuIHJlc3VsdC5Ub0ltbXV0YWJsZUxpc3QoKTsKICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgSW1tdXRhYmxlTGlzdDxTeW50YXhOb2RlPiBHZXREZXNjZW5kYW50cyhTeW50YXhOb2RlIG5vZGUpCiAgICAgICAgewogICAgICAgICAgICB2YXIgcmVzdWx0ID0gbmV3IExpc3Q8U3ludGF4Tm9kZT4oKTsKICAgICAgICAgICAgVHJhdmVyc2VUcmVlKG5vZGUsIGRlc2NlbmRhbnQgPT4KICAgICAgICAgICAgewogICAgICAgICAgICAgICAgcmVzdWx0LkFkZChkZXNjZW5kYW50KTsKICAgICAgICAgICAgfSk7CiAgICAgICAgICAgIHJldHVybiByZXN1bHQuVG9JbW11dGFibGVMaXN0KCk7CiAgICAgICAgfQoKICAgICAgICBwdWJsaWMgc3RhdGljIHZvaWQgVHJhdmVyc2VUcmVlKFN5bnRheE5vZGUgc3ludGF4Tm9kZSwgQWN0aW9uPFN5bnRheE5vZGU+IHZpc2l0KQogICAgICAgIHsKICAgICAgICAgICAgdmlzaXQoc3ludGF4Tm9kZSk7CiAgICAgICAgICAgIGZvcmVhY2ggKHZhciBjaGlsZCBpbiBzeW50YXhOb2RlLkNoaWxkTm9kZXMoKSkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgVHJhdmVyc2VUcmVlKGNoaWxkLCB2aXNpdCk7CiAgICAgICAgICAgIH0KICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgc3RyaW5nIENvbnZlcnRUb1BhcmFtZXRlcihJUGFyYW1ldGVyU3ltYm9sIHBhcmFtZXRlcikKICAgICAgICB7CiAgICAgICAgICAgIHZhciBtb2RpZmllciA9IHN0cmluZy5Kb2luKCIgIiwgcGFyYW1ldGVyLkN1c3RvbU1vZGlmaWVycyk7CiAgICAgICAgICAgIGlmIChwYXJhbWV0ZXIuUmVmS2luZCA9PSBSZWZLaW5kLkluKQogICAgICAgICAgICAgICAgbW9kaWZpZXIgKz0gIiBpbiI7CiAgICAgICAgICAgIGlmIChwYXJhbWV0ZXIuUmVmS2luZCA9PSBSZWZLaW5kLk91dCkKICAgICAgICAgICAgICAgIG1vZGlmaWVyICs9ICIgb3V0IjsKICAgICAgICAgICAgaWYgKHBhcmFtZXRlci5SZWZLaW5kID09IFJlZktpbmQuUmVmKQogICAgICAgICAgICAgICAgbW9kaWZpZXIgKz0gIiByZWYiOwogICAgICAgICAgICByZXR1cm4gJCJ7bW9kaWZpZXJ9IHtwYXJhbWV0ZXIuVHlwZX0ge3BhcmFtZXRlci5OYW1lfSI7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHB1YmxpYyBzdGF0aWMgc3RyaW5nIENvbnZlcnRUb0FyZ3VtZW50KElQYXJhbWV0ZXJTeW1ib2wgcGFyYW1ldGVyKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIG1vZGlmaWVyID0gc3RyaW5nLkpvaW4oIiAiLCBwYXJhbWV0ZXIuQ3VzdG9tTW9kaWZpZXJzKTsKICAgICAgICAgICAgaWYgKHBhcmFtZXRlci5SZWZLaW5kID09IFJlZktpbmQuSW4pCiAgICAgICAgICAgICAgICBtb2RpZmllciArPSAiIGluIjsKICAgICAgICAgICAgaWYgKHBhcmFtZXRlci5SZWZLaW5kID09IFJlZktpbmQuT3V0KQogICAgICAgICAgICAgICAgbW9kaWZpZXIgKz0gIiBvdXQiOwogICAgICAgICAgICBpZiAocGFyYW1ldGVyLlJlZktpbmQgPT0gUmVmS2luZC5SZWYpCiAgICAgICAgICAgICAgICBtb2RpZmllciArPSAiIHJlZiI7CiAgICAgICAgICAgIHJldHVybiAkInttb2RpZmllcn0ge3BhcmFtZXRlci5OYW1lfSI7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHB1YmxpYyBzdGF0aWMgSUVudW1lcmFibGU8c3RyaW5nPiBHZXRVc2luZ3MoSVR5cGVTeW1ib2wgdHlwZVN5bWJvbCkKICAgICAgICB7CiAgICAgICAgICAgIHZhciByZXN1bHRzID0gbmV3IExpc3Q8c3RyaW5nPigpOwogICAgICAgICAgICBHZXRVc2luZ3ModHlwZVN5bWJvbCwgcmVzdWx0cyk7CiAgICAgICAgICAgIHJldHVybiByZXN1bHRzOwogICAgICAgIH0KCiAgICAgICAgcHJpdmF0ZSBzdGF0aWMgdm9pZCBHZXRVc2luZ3MoSVR5cGVTeW1ib2wgdHlwZVN5bWJvbCwgTGlzdDxzdHJpbmc+IHJlc3VsdHMpCiAgICAgICAgewogICAgICAgICAgICBpZiAodHlwZVN5bWJvbC5Db250YWluaW5nTmFtZXNwYWNlICE9IG51bGwpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHZhciByZXN1bHQgPSBzdHJpbmcuSm9pbigiLiIsIHR5cGVTeW1ib2wuQ29udGFpbmluZ05hbWVzcGFjZS5Db25zdGl0dWVudE5hbWVzcGFjZXMpOwogICAgICAgICAgICAgICAgaWYgKHJlc3VsdCAhPSAiPGdsb2JhbCBuYW1lc3BhY2U+IikKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICByZXN1bHRzLkFkZCgidXNpbmcgIiArIHN0cmluZy5Kb2luKCIuIiwgdHlwZVN5bWJvbC5Db250YWluaW5nTmFtZXNwYWNlLkNvbnN0aXR1ZW50TmFtZXNwYWNlcykgKyAiOyIpOwogICAgICAgICAgICAgICAgfQogICAgICAgICAgICB9CgogICAgICAgICAgICBpZiAodHlwZVN5bWJvbCBpcyBJTmFtZWRUeXBlU3ltYm9sIG5hbWVkVHlwZVN5bWJvbCkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgZm9yZWFjaCAodmFyIHR5cGVBcmd1bWVudCBpbiBuYW1lZFR5cGVTeW1ib2wuVHlwZUFyZ3VtZW50cykKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICBHZXRVc2luZ3ModHlwZUFyZ3VtZW50LCByZXN1bHRzKTsKICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgfQogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgc3RhdGljIEltbXV0YWJsZURpY3Rpb25hcnk8c3RyaW5nLCBJbW11dGFibGVMaXN0PElTeW1ib2w+PiBHZXRNZW1iZXJzR3JvdXBlZEJ5RGVjbGFyaW5nVHlwZShJbnRlcmZhY2VEZWNsYXJhdGlvblN5bnRheCBzeW50YXgsIFNlbWFudGljTW9kZWwgc2VtYW50aWNNb2RlbCkKICAgICAgICB7CiAgICAgICAgICAgIHZhciBzeW1ib2wgPSBzZW1hbnRpY01vZGVsLkdldERlY2xhcmVkU3ltYm9sKHN5bnRheCk7CiAgICAgICAgICAgIHJldHVybiBHZXRNZW1iZXJzR3JvdXBlZEJ5RGVjbGFyaW5nVHlwZShzeW1ib2wpOwogICAgICAgIH0KCiAgICAgICAgcHVibGljIHN0YXRpYyBJbW11dGFibGVEaWN0aW9uYXJ5PHN0cmluZywgSW1tdXRhYmxlTGlzdDxJU3ltYm9sPj4gR2V0TWVtYmVyc0dyb3VwZWRCeURlY2xhcmluZ1R5cGUoSU5hbWVkVHlwZVN5bWJvbCBzeW1ib2wpCiAgICAgICAgewogICAgICAgICAgICB2YXIgcmVzdWx0cyA9IG5ldyBEaWN0aW9uYXJ5PHN0cmluZywgSW1tdXRhYmxlTGlzdDxJU3ltYm9sPj4oKTsKICAgICAgICAgICAgR2V0TWVtYmVyc0dyb3VwZWRCeURlY2xhcmluZ1R5cGUoc3ltYm9sLCByZXN1bHRzKTsKICAgICAgICAgICAgcmV0dXJuIHJlc3VsdHMuVG9JbW11dGFibGVEaWN0aW9uYXJ5KCk7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHByaXZhdGUgc3RhdGljIHZvaWQgR2V0TWVtYmVyc0dyb3VwZWRCeURlY2xhcmluZ1R5cGUoSU5hbWVkVHlwZVN5bWJvbCBzeW1ib2wsIERpY3Rpb25hcnk8c3RyaW5nLCBJbW11dGFibGVMaXN0PElTeW1ib2w+PiByZXN1bHRzKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIHR5cGVBcmd1bWVudHMgPSBzeW1ib2wuVHlwZUFyZ3VtZW50cy5MZW5ndGggPT0gMCA/ICIiIDogJCI8e3N0cmluZy5Kb2luKCIsICIsIHN5bWJvbC5UeXBlQXJndW1lbnRzKX0+IjsKICAgICAgICAgICAgdmFyIGtleSA9ICQie3N5bWJvbC5OYW1lfXt0eXBlQXJndW1lbnRzfSI7CiAgICAgICAgICAgIGlmIChyZXN1bHRzLkNvbnRhaW5zS2V5KGtleSkpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHJldHVybjsKICAgICAgICAgICAgfQoKICAgICAgICAgICAgcmVzdWx0cy5BZGQoa2V5LCBzeW1ib2wuR2V0TWVtYmVycygpLlRvSW1tdXRhYmxlTGlzdCgpKTsKICAgICAgICAgICAgCiAgICAgICAgICAgIGZvcmVhY2ggKHZhciBiYXNlSW50ZXJmYWNlIGluIHN5bWJvbC5JbnRlcmZhY2VzKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICBHZXRNZW1iZXJzR3JvdXBlZEJ5RGVjbGFyaW5nVHlwZShiYXNlSW50ZXJmYWNlLCByZXN1bHRzKTsKICAgICAgICAgICAgfQogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgc3RhdGljIGJvb2wgSXNCYXNlQ2xhc3MoSU5hbWVkVHlwZVN5bWJvbCBzdXBlckNsYXNzLCBJTmFtZWRUeXBlU3ltYm9sIGJhc2VDbGFzcykKICAgICAgICB7CiAgICAgICAgICAgIHZhciBiYXNlQ2xhc3NlcyA9IG5ldyBMaXN0PElOYW1lZFR5cGVTeW1ib2w+KCk7CiAgICAgICAgICAgIEdldEJhc2VDbGFzc2VzKHN1cGVyQ2xhc3MsIGJhc2VDbGFzc2VzKTsKICAgICAgICAgICAgcmV0dXJuIGJhc2VDbGFzc2VzLkFueSh4ID0+IHguVG9TdHJpbmcoKSA9PSBiYXNlQ2xhc3MuVG9TdHJpbmcoKSk7CiAgICAgICAgfQoKICAgICAgICBwdWJsaWMgc3RhdGljIHN0cmluZyBHZXRXaXRob3V0VHlwZUFyZ3VtZW50cyhzdHJpbmcgdHlwZSkKICAgICAgICB7CiAgICAgICAgICAgIHZhciBpZHggPSB0eXBlLkluZGV4T2YoJzwnKTsKICAgICAgICAgICAgaWYgKGlkeCA+PSAwKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICByZXR1cm4gdHlwZS5TdWJzdHJpbmcoMCwgaWR4KTsKICAgICAgICAgICAgfQoKICAgICAgICAgICAgcmV0dXJuIHR5cGU7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHB1YmxpYyBzdGF0aWMgSVJlYWRPbmx5TGlzdDxJTmFtZWRUeXBlU3ltYm9sPiBHZXRCYXNlQ2xhc3NlcyhJTmFtZWRUeXBlU3ltYm9sIHN1cGVyQ2xhc3MpCiAgICAgICAgewogICAgICAgICAgICB2YXIgYmFzZUNsYXNzZXMgPSBuZXcgTGlzdDxJTmFtZWRUeXBlU3ltYm9sPigpOwogICAgICAgICAgICBHZXRCYXNlQ2xhc3NlcyhzdXBlckNsYXNzLCBiYXNlQ2xhc3Nlcyk7CiAgICAgICAgICAgIHJldHVybiBiYXNlQ2xhc3NlczsKICAgICAgICB9CiAgICAgICAgCiAgICAgICAgcHJpdmF0ZSBzdGF0aWMgdm9pZCBHZXRCYXNlQ2xhc3NlcyhJTmFtZWRUeXBlU3ltYm9sIHN1cGVyQ2xhc3MsIExpc3Q8SU5hbWVkVHlwZVN5bWJvbD4gcmVzdWx0KQogICAgICAgIHsKICAgICAgICAgICAgcmVzdWx0LkFkZChzdXBlckNsYXNzKTsKICAgICAgICAgICAgaWYgKHN1cGVyQ2xhc3M/LkJhc2VUeXBlPy5UeXBlS2luZCA9PSBUeXBlS2luZC5DbGFzcykKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgR2V0QmFzZUNsYXNzZXMoc3VwZXJDbGFzcy5CYXNlVHlwZSwgcmVzdWx0KTsKICAgICAgICAgICAgfQogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgc3RhdGljIGJvb2wgSXNCYXNlSW50ZXJmYWNlKElOYW1lZFR5cGVTeW1ib2wgc3VwZXJJbnRlcmZhY2UsIElOYW1lZFR5cGVTeW1ib2wgYmFzZUludGVyZmFjZSkKICAgICAgICB7CiAgICAgICAgICAgIHZhciBiYXNlSW50ZXJmYWNlcyA9IG5ldyBMaXN0PElOYW1lZFR5cGVTeW1ib2w+KCk7CiAgICAgICAgICAgIEdldEJhc2VJbnRlcmZhY2VzKHN1cGVySW50ZXJmYWNlLCBiYXNlSW50ZXJmYWNlcyk7CiAgICAgICAgICAgIHJldHVybiBiYXNlSW50ZXJmYWNlcy5BbnkoeCA9PiB4LlRvU3RyaW5nKCkgPT0gYmFzZUludGVyZmFjZS5Ub1N0cmluZygpKTsKICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgSVJlYWRPbmx5TGlzdDxJTmFtZWRUeXBlU3ltYm9sPiBHZXRCYXNlSW50ZXJmYWNlcyhJTmFtZWRUeXBlU3ltYm9sIHN1cGVySW50ZXJmYWNlKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIGJhc2VJbnRlcmZhY2VzID0gbmV3IExpc3Q8SU5hbWVkVHlwZVN5bWJvbD4oKTsKICAgICAgICAgICAgR2V0QmFzZUludGVyZmFjZXMoc3VwZXJJbnRlcmZhY2UsIGJhc2VJbnRlcmZhY2VzKTsKICAgICAgICAgICAgcmV0dXJuIGJhc2VJbnRlcmZhY2VzOwogICAgICAgIH0KICAgICAgICAKICAgICAgICBwcml2YXRlIHN0YXRpYyB2b2lkIEdldEJhc2VJbnRlcmZhY2VzKElOYW1lZFR5cGVTeW1ib2wgc3VwZXJJbnRlcmZhY2UsIExpc3Q8SU5hbWVkVHlwZVN5bWJvbD4gcmVzdWx0KQogICAgICAgIHsKICAgICAgICAgICAgcmVzdWx0LkFkZChzdXBlckludGVyZmFjZSk7CiAgICAgICAgICAgIGZvcmVhY2ggKHZhciBiYXNlVHlwZSBpbiBzdXBlckludGVyZmFjZS5JbnRlcmZhY2VzKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICBHZXRCYXNlSW50ZXJmYWNlcyhiYXNlVHlwZSwgcmVzdWx0KTsKICAgICAgICAgICAgfQogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgc3RhdGljIHN0cmluZyBHZW5lcmF0ZVZhcmlhYmxlTmFtZShzdHJpbmcgdHlwZU5hbWUsIGJvb2wgaXNJbnRlcmZhY2UpCiAgICAgICAgewogICAgICAgICAgICB2YXIgcmVzdWx0ID0gaXNJbnRlcmZhY2UgPyB0eXBlTmFtZS5TdWJzdHJpbmcoMSkgOiB0eXBlTmFtZTsKICAgICAgICAgICAgaWYgKHJlc3VsdC5Db250YWlucygiPCIpKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICByZXN1bHQgPSByZXN1bHQuU3Vic3RyaW5nKDAsIHJlc3VsdC5JbmRleE9mKCc8JykpOwogICAgICAgICAgICB9CgogICAgICAgICAgICByZXR1cm4gcmVzdWx0LkNhbWVsaXplKCk7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIC8vLyA8c3VtbWFyeT4KICAgICAgICAvLy8gSWYgeW91IHBhc3MgaW4gbmV3W117MCwgMX0gYW5kIG5ld1tdezAsIDF9IHlvdSdsbCBnZXQgbmV3W117MCwgMH0sIG5ld1tdezAsIDF9LCBuZXdbXXsxLCAwfSwgbmV3W117MSwgMX0gb3V0LgogICAgICAgIC8vLyA8L3N1bW1hcnk+CiAgICAgICAgLy8vIDx0eXBlcGFyYW0gbmFtZT0iVEl0ZW0iPlRoZSB0eXBlIG9mIGl0ZW0gaW4gdGhlIGVudW1lcmFibGVzLjwvdHlwZXBhcmFtPgogICAgICAgIC8vLyA8cGFyYW0gbmFtZT0iZW51bWVyYWJsZXMiPlRoZSBlbnVtZXJhYmxlcyB0aGF0IHdpbGwgYmUgdXNlZCB0byBjYWxjdWxhdGUgY29tYmluYXRpb25zPC9wYXJhbT4KICAgICAgICAvLy8gPHJldHVybnM+QSBsYXppbHktY29tcHV0ZWQgZW51bWVyYWJsZSB3aGVyZSBlYWNoIHJldHVybiBpdGVtIGlzIGEgY29tYmluYXRpb248L3JldHVybnM+CiAgICAgICAgcHVibGljIHN0YXRpYyBJRW51bWVyYWJsZTxJbW11dGFibGVMaXN0PFRJdGVtPj4gQ2FsY0NvbWJpbmF0aW9uc09mT25lRnJvbUVhY2g8VEl0ZW0+KAogICAgICAgICAgICBJRW51bWVyYWJsZTxJRW51bWVyYWJsZTxUSXRlbT4+IGVudW1lcmFibGVzICkgewogICAgICAgICAgICB2YXIgZW51bWVyYXRvcnMgPSBlbnVtZXJhYmxlcy5TZWxlY3QoIGVudW1lcmFibGUgPT4gVHVwbGUuQ3JlYXRlKCBlbnVtZXJhYmxlLCBlbnVtZXJhYmxlLkdldEVudW1lcmF0b3IoKSApICkuVG9MaXN0KCk7CgogICAgICAgICAgICBmb3JlYWNoICggdmFyIGVudW1lcmF0b3IgaW4gZW51bWVyYXRvcnMgKSB7CiAgICAgICAgICAgICAgICBpZiAoICFlbnVtZXJhdG9yLkl0ZW0yLk1vdmVOZXh0KCkgKSB7IHRocm93IG5ldyBBcmd1bWVudEV4Y2VwdGlvbiggIkFuIGVtcHR5IGVudW1lcmFibGUgd2FzIHNwZWNpZmllZCIgKTsKICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgfQoKICAgICAgICAgICAgeWllbGQgcmV0dXJuIGVudW1lcmF0b3JzLlNlbGVjdCggdCA9PiB0Lkl0ZW0yLkN1cnJlbnQgKS5Ub0ltbXV0YWJsZUxpc3QoKTsKCiAgICAgICAgICAgIHdoaWxlICh0cnVlKSB7CiAgICAgICAgICAgICAgICB2YXIgaSA9IDA7CiAgICAgICAgICAgICAgICB3aGlsZSh0cnVlKSB7CiAgICAgICAgICAgICAgICAgICAgaWYgKCBpID09IGVudW1lcmF0b3JzLkNvdW50ICkgewogICAgICAgICAgICAgICAgICAgICAgICB5aWVsZCBicmVhazsKICAgICAgICAgICAgICAgICAgICB9CgogICAgICAgICAgICAgICAgICAgIGlmICggIWVudW1lcmF0b3JzW2ldLkl0ZW0yLk1vdmVOZXh0KCkgKSB7CiAgICAgICAgICAgICAgICAgICAgICAgIGVudW1lcmF0b3JzW2ldID0gVHVwbGUuQ3JlYXRlKCBlbnVtZXJhdG9yc1tpXS5JdGVtMSwKICAgICAgICAgICAgICAgICAgICAgICAgICAgIGVudW1lcmF0b3JzW2ldLkl0ZW0xLkdldEVudW1lcmF0b3IoKSApOwogICAgICAgICAgICAgICAgICAgICAgICBlbnVtZXJhdG9yc1tpXS5JdGVtMi5Nb3ZlTmV4dCgpOwogICAgICAgICAgICAgICAgICAgIH0gZWxzZSB7CiAgICAgICAgICAgICAgICAgICAgICAgIGJyZWFrOwogICAgICAgICAgICAgICAgICAgIH0KCiAgICAgICAgICAgICAgICAgICAgaSsrOwogICAgICAgICAgICAgICAgfQoKICAgICAgICAgICAgICAgIHlpZWxkIHJldHVybiBlbnVtZXJhdG9ycy5TZWxlY3QoIHQgPT4gdC5JdGVtMi5DdXJyZW50ICkuVG9JbW11dGFibGVMaXN0KCk7CiAgICAgICAgICAgIH0KICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgc3RyaW5nIENvbnZlcnRUb1R5cGVQYXJhbWV0ZXJzKEltbXV0YWJsZUFycmF5PElUeXBlUGFyYW1ldGVyU3ltYm9sPiBtZXRob2REZWNsYXJhdGlvblR5cGVQYXJhbWV0ZXJzKQogICAgICAgIHsKICAgICAgICAgICAgaWYgKG1ldGhvZERlY2xhcmF0aW9uVHlwZVBhcmFtZXRlcnMuTGVuZ3RoID09IDApCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHJldHVybiAiIjsKICAgICAgICAgICAgfQoKICAgICAgICAgICAgcmV0dXJuCiAgICAgICAgICAgICAgICAkIjx7c3RyaW5nLkpvaW4oIiwgIiwgbWV0aG9kRGVjbGFyYXRpb25UeXBlUGFyYW1ldGVycy5TZWxlY3QodHlwZVBhcmFtZXRlciA9PiB0eXBlUGFyYW1ldGVyLk5hbWUpKX0+IjsKICAgICAgICB9CiAgICB9Cn0="
    },
    "nloum/CodeIO@d46184b15ab068ef7e589707a5962333f1e40cb3:src/FluentSourceGenerators/Utilities.cs": {
      "sha256": "4520135111278f0029a45212eb479089867424f74f14ecc7b518d8d44ca89dd8",
      "base64": "dXNpbmcgU3lzdGVtOwp1c2luZyBTeXN0ZW0uQ29sbGVjdGlvbnMuR2VuZXJpYzsKdXNpbmcgU3lzdGVtLkNvbGxlY3Rpb25zLkltbXV0YWJsZTsKdXNpbmcgU3lzdGVtLkxpbnE7CnVzaW5nIFN5c3RlbS5SdW50aW1lLkNvbXBpbGVyU2VydmljZXM7CnVzaW5nIFN5c3RlbS5UZXh0LlJlZ3VsYXJFeHByZXNzaW9uczsKdXNpbmcgTWljcm9zb2Z0LkNvZGVBbmFseXNpczsKdXNpbmcgTWljcm9zb2Z0LkNvZGVBbmFseXNpcy5DU2hhcnA7CnVzaW5nIE1pY3Jvc29mdC5Db2RlQW5hbHlzaXMuQ1NoYXJwLlN5bnRheDsKCm5hbWVzcGFjZSBGbHVlbnRTb3VyY2VHZW5lcmF0b3JzCnsKICAgIHB1YmxpYyBzdGF0aWMgY2xhc3MgVXRpbGl0aWVzCiAgICB7CiAgICAgICAgcHVibGljIHN0YXRpYyB2b2lkIFRocm93RXhjZXB0aW9uKHN0cmluZyBlcnJvck1lc3NhZ2UsIFtDYWxsZXJGaWxlUGF0aF0gc3RyaW5nIGNhbGxlckZpbGVQYXRoID0gbnVsbCwKICAgICAgICAgICAgW0NhbGxlckxpbmVOdW1iZXJdIGludCBjYWxsZXJMaW5lTnVtYmVyID0gMCwgW0NhbGxlck1lbWJlck5hbWVdIHN0cmluZyBjYWxsZXJNZW1iZXJOYW1lID0gbnVsbCkKICAgICAgICB7CiAgICAgICAgICAgIHRocm93IG5ldyBJbnZhbGlkT3BlcmF0aW9uRXhjZXB0aW9uKGVycm9yTWVzc2FnZSArCiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICQiICh7Y2FsbGVyRmlsZVBhdGh9OntjYWxsZXJMaW5lTnVtYmVyfSBpbiB7Y2FsbGVyTWVtYmVyTmFtZX0pIik7CiAgICAgICAgfSAKICAgIAogICAgICAgIC8vLyA8c3VtbWFyeT4KICAgICAgICAvLy8gQnkgZGVmYXVsdCwgcGFzY2FsaXplIGNvbnZlcnRzIHN0cmluZ3MgdG8gVXBwZXJDYW1lbENhc2UgYWxzbyByZW1vdmluZyB1bmRlcnNjb3JlcwogICAgICAgIC8vLyA8L3N1bW1hcnk+CiAgICAgICAgLy8vIDxwYXJhbSBuYW1lPSJpbnB1dCI+PC9wYXJhbT4KICAgICAgICAvLy8gPHJldHVybnM+PC9yZXR1cm5zPgogICAgICAgIHB1YmxpYyBzdGF0aWMgc3RyaW5nIFBhc2NhbGl6ZSh0aGlzIHN0cmluZyBpbnB1dCkKICAgICAgICB7ICAgCiAgICAgICAgICAgIHJldHVybiBSZWdleC5SZXBsYWNlKGlucHV0LCAiKD86XnxffCArKSguKSIsIG1hdGNoID0+IG1hdGNoLkdyb3Vwc1sxXS5WYWx1ZS5Ub1VwcGVyKCkpOwogICAgICAgIH0KCiAgICAgICAgLy8vIDxzdW1tYXJ5PgogICAgICAgIC8vLyBTYW1lIGFzIFBhc2NhbGl6ZSBleGNlcHQgdGhhdCB0aGUgZmlyc3QgY2hhcmFjdGVyIGlzIGxvd2VyIGNhc2UKICAgICAgICAvLy8gPC9zdW1tYXJ5PgogICAgICAgIC8vLyA8cGFyYW0gbmFtZT0iaW5wdXQiPjwvcGFyYW0+CiAgICAgICAgLy8vIDxyZXR1cm5zPjwvcmV0dXJucz4KICAgICAgICBwdWJsaWMgc3RhdGljIHN0cmluZyBDYW1lbGl6ZSh0aGlzIHN0cmluZyBpbnB1dCkKICAgICAgICB7CiAgICAgICAgICAgIHZhciB3b3JkID0gaW5wdXQuUGFzY2FsaXplKCk7CiAgICAgICAgICAgIHJldHVybiB3b3JkLkxlbmd0aCA+IDAgPyB3b3JkLlN1YnN0cmluZygwLCAxKS5Ub0xvd2VyKCkgKyB3b3JkLlN1YnN0cmluZygxKSA6IHdvcmQ7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHB1YmxpYyBzdGF0aWMgU3ludGF4Tm9kZT8gRmluZFBhcmVudE9mS2luZChTeW50YXhOb2RlIGNoaWxkLCBTeW50YXhLaW5kIHN5bnRheEtpbmQpCiAgICAgICAgewogICAgICAgICAgICBpZiAoY2hpbGQuSXNLaW5kKHN5bnRheEtpbmQpKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICByZXR1cm4gY2hpbGQ7CiAgICAgICAgICAgIH0KCiAgICAgICAgICAgIGlmIChjaGlsZC5QYXJlbnQgIT0gbnVsbCkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgcmV0dXJuIEZpbmRQYXJlbnRPZktpbmQoY2hpbGQuUGFyZW50LCBzeW50YXhLaW5kKTsKICAgICAgICAgICAgfQoKICAgICAgICAgICAgcmV0dXJuIG51bGw7CiAgICAgICAgfQoKICAgICAgICBwdWJsaWMgc3RhdGljIEltbXV0YWJsZUxpc3Q8VD4gR2V0RGVzY2VuZGFudHNPZlR5cGU8VD4oU3ludGF4Tm9kZSBub2RlKSB3aGVyZSBUIDogU3ludGF4Tm9kZQogICAgICAgIHsKICAgICAgICAgICAgdmFyIHJlc3VsdCA9IG5ldyBMaXN0PFQ+KCk7CiAgICAgICAgICAgIFRyYXZlcnNlVHJlZShub2RlLCBkZXNjZW5kYW50ID0+CiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIGlmIChkZXNjZW5kYW50IGlzIFQgdCkKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICByZXN1bHQuQWRkKHQpOwogICAgICAgICAgICAgICAgfQogICAgICAgICAgICB9KTsKICAgICAgICAgICAgcmV0dXJuIHJlc3VsdC5Ub0ltbXV0YWJsZUxpc3QoKTsKICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgSW1tdXRhYmxlTGlzdDxTeW50YXhOb2RlPiBHZXREZXNjZW5kYW50cyhTeW50YXhOb2RlIG5vZGUpCiAgICAgICAgewogICAgICAgICAgICB2YXIgcmVzdWx0ID0gbmV3IExpc3Q8U3ludGF4Tm9kZT4oKTsKICAgICAgICAgICAgVHJhdmVyc2VUcmVlKG5vZGUsIGRlc2NlbmRhbnQgPT4KICAgICAgICAgICAgewogICAgICAgICAgICAgICAgcmVzdWx0LkFkZChkZXNjZW5kYW50KTsKICAgICAgICAgICAgfSk7CiAgICAgICAgICAgIHJldHVybiByZXN1bHQuVG9JbW11dGFibGVMaXN0KCk7CiAgICAgICAgfQoKICAgICAgICBwdWJsaWMgc3RhdGljIHZvaWQgVHJhdmVyc2VUcmVlKFN5bnRheE5vZGUgc3ludGF4Tm9kZSwgQWN0aW9uPFN5bnRheE5vZGU+IHZpc2l0KQogICAgICAgIHsKICAgICAgICAgICAgdmlzaXQoc3ludGF4Tm9kZSk7CiAgICAgICAgICAgIGZvcmVhY2ggKHZhciBjaGlsZCBpbiBzeW50YXhOb2RlLkNoaWxkTm9kZXMoKSkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgVHJhdmVyc2VUcmVlKGNoaWxkLCB2aXNpdCk7CiAgICAgICAgICAgIH0KICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgc3RyaW5nIENvbnZlcnRUb1BhcmFtZXRlcihJUGFyYW1ldGVyU3ltYm9sIHBhcmFtZXRlcikKICAgICAgICB7CiAgICAgICAgICAgIHZhciBtb2RpZmllciA9IHN0cmluZy5Kb2luKCIgIiwgcGFyYW1ldGVyLkN1c3RvbU1vZGlmaWVycyk7CiAgICAgICAgICAgIGlmIChwYXJhbWV0ZXIuUmVmS2luZCA9PSBSZWZLaW5kLkluKQogICAgICAgICAgICAgICAgbW9kaWZpZXIgKz0gIiBpbiI7CiAgICAgICAgICAgIGlmIChwYXJhbWV0ZXIuUmVmS2luZCA9PSBSZWZLaW5kLk91dCkKICAgICAgICAgICAgICAgIG1vZGlmaWVyICs9ICIgb3V0IjsKICAgICAgICAgICAgaWYgKHBhcmFtZXRlci5SZWZLaW5kID09IFJlZktpbmQuUmVmKQogICAgICAgICAgICAgICAgbW9kaWZpZXIgKz0gIiByZWYiOwogICAgICAgICAgICByZXR1cm4gJCJ7bW9kaWZpZXJ9IHtwYXJhbWV0ZXIuVHlwZX0ge3BhcmFtZXRlci5OYW1lfSI7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHB1YmxpYyBzdGF0aWMgc3RyaW5nIENvbnZlcnRUb0FyZ3VtZW50KElQYXJhbWV0ZXJTeW1ib2wgcGFyYW1ldGVyKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIG1vZGlmaWVyID0gc3RyaW5nLkpvaW4oIiAiLCBwYXJhbWV0ZXIuQ3VzdG9tTW9kaWZpZXJzKTsKICAgICAgICAgICAgaWYgKHBhcmFtZXRlci5SZWZLaW5kID09IFJlZktpbmQuSW4pCiAgICAgICAgICAgICAgICBtb2RpZmllciArPSAiIGluIjsKICAgICAgICAgICAgaWYgKHBhcmFtZXRlci5SZWZLaW5kID09IFJlZktpbmQuT3V0KQogICAgICAgICAgICAgICAgbW9kaWZpZXIgKz0gIiBvdXQiOwogICAgICAgICAgICBpZiAocGFyYW1ldGVyLlJlZktpbmQgPT0gUmVmS2luZC5SZWYpCiAgICAgICAgICAgICAgICBtb2RpZmllciArPSAiIHJlZiI7CiAgICAgICAgICAgIHJldHVybiAkInttb2RpZmllcn0ge3BhcmFtZXRlci5OYW1lfSI7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHB1YmxpYyBzdGF0aWMgSUVudW1lcmFibGU8c3RyaW5nPiBHZXRVc2luZ3MoSVR5cGVTeW1ib2wgdHlwZVN5bWJvbCkKICAgICAgICB7CiAgICAgICAgICAgIHZhciByZXN1bHRzID0gbmV3IExpc3Q8c3RyaW5nPigpOwogICAgICAgICAgICBHZXRVc2luZ3ModHlwZVN5bWJvbCwgcmVzdWx0cyk7CiAgICAgICAgICAgIHJldHVybiByZXN1bHRzOwogICAgICAgIH0KCiAgICAgICAgcHJpdmF0ZSBzdGF0aWMgdm9pZCBHZXRVc2luZ3MoSVR5cGVTeW1ib2wgdHlwZVN5bWJvbCwgTGlzdDxzdHJpbmc+IHJlc3VsdHMpCiAgICAgICAgewogICAgICAgICAgICBpZiAodHlwZVN5bWJvbC5Db250YWluaW5nTmFtZXNwYWNlICE9IG51bGwpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHZhciByZXN1bHQgPSBzdHJpbmcuSm9pbigiLiIsIHR5cGVTeW1ib2wuQ29udGFpbmluZ05hbWVzcGFjZS5Db25zdGl0dWVudE5hbWVzcGFjZXMpOwogICAgICAgICAgICAgICAgaWYgKHJlc3VsdCAhPSAiPGdsb2JhbCBuYW1lc3BhY2U+IikKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICByZXN1bHRzLkFkZCgidXNpbmcgIiArIHN0cmluZy5Kb2luKCIuIiwgdHlwZVN5bWJvbC5Db250YWluaW5nTmFtZXNwYWNlLkNvbnN0aXR1ZW50TmFtZXNwYWNlcykgKyAiOyIpOwogICAgICAgICAgICAgICAgfQogICAgICAgICAgICB9CgogICAgICAgICAgICBpZiAodHlwZVN5bWJvbCBpcyBJTmFtZWRUeXBlU3ltYm9sIG5hbWVkVHlwZVN5bWJvbCkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgZm9yZWFjaCAodmFyIHR5cGVBcmd1bWVudCBpbiBuYW1lZFR5cGVTeW1ib2wuVHlwZUFyZ3VtZW50cykKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICBHZXRVc2luZ3ModHlwZUFyZ3VtZW50LCByZXN1bHRzKTsKICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgfQogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgc3RhdGljIEltbXV0YWJsZURpY3Rpb25hcnk8c3RyaW5nLCBJbW11dGFibGVMaXN0PElTeW1ib2w+PiBHZXRNZW1iZXJzR3JvdXBlZEJ5RGVjbGFyaW5nVHlwZShJbnRlcmZhY2VEZWNsYXJhdGlvblN5bnRheCBzeW50YXgsIFNlbWFudGljTW9kZWwgc2VtYW50aWNNb2RlbCkKICAgICAgICB7CiAgICAgICAgICAgIHZhciBzeW1ib2wgPSBzZW1hbnRpY01vZGVsLkdldERlY2xhcmVkU3ltYm9sKHN5bnRheCk7CiAgICAgICAgICAgIHJldHVybiBHZXRNZW1iZXJzR3JvdXBlZEJ5RGVjbGFyaW5nVHlwZShzeW1ib2wpOwogICAgICAgIH0KCiAgICAgICAgcHVibGljIHN0YXRpYyBJbW11dGFibGVEaWN0aW9uYXJ5PHN0cmluZywgSW1tdXRhYmxlTGlzdDxJU3ltYm9sPj4gR2V0TWVtYmVyc0dyb3VwZWRCeURlY2xhcmluZ1R5cGUoSU5hbWVkVHlwZVN5bWJvbCBzeW1ib2wpCiAgICAgICAgewogICAgICAgICAgICB2YXIgcmVzdWx0cyA9IG5ldyBEaWN0aW9uYXJ5PHN0cmluZywgSW1tdXRhYmxlTGlzdDxJU3ltYm9sPj4oKTsKICAgICAgICAgICAgR2V0TWVtYmVyc0dyb3VwZWRCeURlY2xhcmluZ1R5cGUoc3ltYm9sLCByZXN1bHRzKTsKICAgICAgICAgICAgcmV0dXJuIHJlc3VsdHMuVG9JbW11dGFibGVEaWN0aW9uYXJ5KCk7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHByaXZhdGUgc3RhdGljIHZvaWQgR2V0TWVtYmVyc0dyb3VwZWRCeURlY2xhcmluZ1R5cGUoSU5hbWVkVHlwZVN5bWJvbCBzeW1ib2wsIERpY3Rpb25hcnk8c3RyaW5nLCBJbW11dGFibGVMaXN0PElTeW1ib2w+PiByZXN1bHRzKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIHR5cGVBcmd1bWVudHMgPSBzeW1ib2wuVHlwZUFyZ3VtZW50cy5MZW5ndGggPT0gMCA/ICIiIDogJCI8e3N0cmluZy5Kb2luKCIsICIsIHN5bWJvbC5UeXBlQXJndW1lbnRzKX0+IjsKICAgICAgICAgICAgdmFyIGtleSA9ICQie3N5bWJvbC5OYW1lfXt0eXBlQXJndW1lbnRzfSI7CiAgICAgICAgICAgIGlmIChyZXN1bHRzLkNvbnRhaW5zS2V5KGtleSkpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHJldHVybjsKICAgICAgICAgICAgfQoKICAgICAgICAgICAgcmVzdWx0cy5BZGQoa2V5LCBzeW1ib2wuR2V0TWVtYmVycygpLlRvSW1tdXRhYmxlTGlzdCgpKTsKICAgICAgICAgICAgCiAgICAgICAgICAgIGZvcmVhY2ggKHZhciBiYXNlSW50ZXJmYWNlIGluIHN5bWJvbC5JbnRlcmZhY2VzKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICBHZXRNZW1iZXJzR3JvdXBlZEJ5RGVjbGFyaW5nVHlwZShiYXNlSW50ZXJmYWNlLCByZXN1bHRzKTsKICAgICAgICAgICAgfQogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgc3RhdGljIGJvb2wgSXNCYXNlQ2xhc3MoSU5hbWVkVHlwZVN5bWJvbCBzdXBlckNsYXNzLCBJTmFtZWRUeXBlU3ltYm9sIGJhc2VDbGFzcykKICAgICAgICB7CiAgICAgICAgICAgIHZhciBiYXNlQ2xhc3NlcyA9IG5ldyBMaXN0PElOYW1lZFR5cGVTeW1ib2w+KCk7CiAgICAgICAgICAgIEdldEJhc2VDbGFzc2VzKHN1cGVyQ2xhc3MsIGJhc2VDbGFzc2VzKTsKICAgICAgICAgICAgcmV0dXJuIGJhc2VDbGFzc2VzLkFueSh4ID0+IHguVG9TdHJpbmcoKSA9PSBiYXNlQ2xhc3MuVG9TdHJpbmcoKSk7CiAgICAgICAgfQoKICAgICAgICBwdWJsaWMgc3RhdGljIHN0cmluZyBHZXRXaXRob3V0VHlwZUFyZ3VtZW50cyhzdHJpbmcgdHlwZSkKICAgICAgICB7CiAgICAgICAgICAgIHZhciBpZHggPSB0eXBlLkluZGV4T2YoJzwnKTsKICAgICAgICAgICAgaWYgKGlkeCA+PSAwKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICByZXR1cm4gdHlwZS5TdWJzdHJpbmcoMCwgaWR4KTsKICAgICAgICAgICAgfQoKICAgICAgICAgICAgcmV0dXJuIHR5cGU7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHB1YmxpYyBzdGF0aWMgSVJlYWRPbmx5TGlzdDxJTmFtZWRUeXBlU3ltYm9sPiBHZXRCYXNlQ2xhc3NlcyhJTmFtZWRUeXBlU3ltYm9sIHN1cGVyQ2xhc3MpCiAgICAgICAgewogICAgICAgICAgICB2YXIgYmFzZUNsYXNzZXMgPSBuZXcgTGlzdDxJTmFtZWRUeXBlU3ltYm9sPigpOwogICAgICAgICAgICBHZXRCYXNlQ2xhc3NlcyhzdXBlckNsYXNzLCBiYXNlQ2xhc3Nlcyk7CiAgICAgICAgICAgIHJldHVybiBiYXNlQ2xhc3NlczsKICAgICAgICB9CiAgICAgICAgCiAgICAgICAgcHJpdmF0ZSBzdGF0aWMgdm9pZCBHZXRCYXNlQ2xhc3NlcyhJTmFtZWRUeXBlU3ltYm9sIHN1cGVyQ2xhc3MsIExpc3Q8SU5hbWVkVHlwZVN5bWJvbD4gcmVzdWx0KQogICAgICAgIHsKICAgICAgICAgICAgcmVzdWx0LkFkZChzdXBlckNsYXNzKTsKICAgICAgICAgICAgaWYgKHN1cGVyQ2xhc3M/LkJhc2VUeXBlPy5UeXBlS2luZCA9PSBUeXBlS2luZC5DbGFzcykKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgR2V0QmFzZUNsYXNzZXMoc3VwZXJDbGFzcy5CYXNlVHlwZSwgcmVzdWx0KTsKICAgICAgICAgICAgfQogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgc3RhdGljIGJvb2wgSXNCYXNlSW50ZXJmYWNlKElOYW1lZFR5cGVTeW1ib2wgc3VwZXJJbnRlcmZhY2UsIElOYW1lZFR5cGVTeW1ib2wgYmFzZUludGVyZmFjZSkKICAgICAgICB7CiAgICAgICAgICAgIHZhciBiYXNlSW50ZXJmYWNlcyA9IG5ldyBMaXN0PElOYW1lZFR5cGVTeW1ib2w+KCk7CiAgICAgICAgICAgIEdldEJhc2VJbnRlcmZhY2VzKHN1cGVySW50ZXJmYWNlLCBiYXNlSW50ZXJmYWNlcyk7CiAgICAgICAgICAgIHJldHVybiBiYXNlSW50ZXJmYWNlcy5BbnkoeCA9PiB4LlRvU3RyaW5nKCkgPT0gYmFzZUludGVyZmFjZS5Ub1N0cmluZygpKTsKICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgSVJlYWRPbmx5TGlzdDxJTmFtZWRUeXBlU3ltYm9sPiBHZXRCYXNlSW50ZXJmYWNlcyhJTmFtZWRUeXBlU3ltYm9sIHN1cGVySW50ZXJmYWNlKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIGJhc2VJbnRlcmZhY2VzID0gbmV3IExpc3Q8SU5hbWVkVHlwZVN5bWJvbD4oKTsKICAgICAgICAgICAgR2V0QmFzZUludGVyZmFjZXMoc3VwZXJJbnRlcmZhY2UsIGJhc2VJbnRlcmZhY2VzKTsKICAgICAgICAgICAgcmV0dXJuIGJhc2VJbnRlcmZhY2VzOwogICAgICAgIH0KICAgICAgICAKICAgICAgICBwcml2YXRlIHN0YXRpYyB2b2lkIEdldEJhc2VJbnRlcmZhY2VzKElOYW1lZFR5cGVTeW1ib2wgc3VwZXJJbnRlcmZhY2UsIExpc3Q8SU5hbWVkVHlwZVN5bWJvbD4gcmVzdWx0KQogICAgICAgIHsKICAgICAgICAgICAgcmVzdWx0LkFkZChzdXBlckludGVyZmFjZSk7CiAgICAgICAgICAgIGZvcmVhY2ggKHZhciBiYXNlVHlwZSBpbiBzdXBlckludGVyZmFjZS5JbnRlcmZhY2VzKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICBHZXRCYXNlSW50ZXJmYWNlcyhiYXNlVHlwZSwgcmVzdWx0KTsKICAgICAgICAgICAgfQogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgc3RhdGljIHN0cmluZyBHZW5lcmF0ZVZhcmlhYmxlTmFtZShzdHJpbmcgdHlwZU5hbWUsIGJvb2wgaXNJbnRlcmZhY2UpCiAgICAgICAgewogICAgICAgICAgICB2YXIgcmVzdWx0ID0gaXNJbnRlcmZhY2UgPyB0eXBlTmFtZS5TdWJzdHJpbmcoMSkgOiB0eXBlTmFtZTsKICAgICAgICAgICAgaWYgKHJlc3VsdC5Db250YWlucygiPCIpKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICByZXN1bHQgPSByZXN1bHQuU3Vic3RyaW5nKDAsIHJlc3VsdC5JbmRleE9mKCc8JykpOwogICAgICAgICAgICB9CgogICAgICAgICAgICByZXR1cm4gcmVzdWx0LkNhbWVsaXplKCk7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIC8vLyA8c3VtbWFyeT4KICAgICAgICAvLy8gSWYgeW91IHBhc3MgaW4gbmV3W117MCwgMX0gYW5kIG5ld1tdezAsIDF9IHlvdSdsbCBnZXQgbmV3W117MCwgMH0sIG5ld1tdezAsIDF9LCBuZXdbXXsxLCAwfSwgbmV3W117MSwgMX0gb3V0LgogICAgICAgIC8vLyA8L3N1bW1hcnk+CiAgICAgICAgLy8vIDx0eXBlcGFyYW0gbmFtZT0iVEl0ZW0iPlRoZSB0eXBlIG9mIGl0ZW0gaW4gdGhlIGVudW1lcmFibGVzLjwvdHlwZXBhcmFtPgogICAgICAgIC8vLyA8cGFyYW0gbmFtZT0iZW51bWVyYWJsZXMiPlRoZSBlbnVtZXJhYmxlcyB0aGF0IHdpbGwgYmUgdXNlZCB0byBjYWxjdWxhdGUgY29tYmluYXRpb25zPC9wYXJhbT4KICAgICAgICAvLy8gPHJldHVybnM+QSBsYXppbHktY29tcHV0ZWQgZW51bWVyYWJsZSB3aGVyZSBlYWNoIHJldHVybiBpdGVtIGlzIGEgY29tYmluYXRpb248L3JldHVybnM+CiAgICAgICAgcHVibGljIHN0YXRpYyBJRW51bWVyYWJsZTxJbW11dGFibGVMaXN0PFRJdGVtPj4gQ2FsY0NvbWJpbmF0aW9uc09mT25lRnJvbUVhY2g8VEl0ZW0+KAogICAgICAgICAgICBJRW51bWVyYWJsZTxJRW51bWVyYWJsZTxUSXRlbT4+IGVudW1lcmFibGVzICkgewogICAgICAgICAgICB2YXIgZW51bWVyYXRvcnMgPSBlbnVtZXJhYmxlcy5TZWxlY3QoIGVudW1lcmFibGUgPT4gVHVwbGUuQ3JlYXRlKCBlbnVtZXJhYmxlLCBlbnVtZXJhYmxlLkdldEVudW1lcmF0b3IoKSApICkuVG9MaXN0KCk7CgogICAgICAgICAgICBmb3JlYWNoICggdmFyIGVudW1lcmF0b3IgaW4gZW51bWVyYXRvcnMgKSB7CiAgICAgICAgICAgICAgICBpZiAoICFlbnVtZXJhdG9yLkl0ZW0yLk1vdmVOZXh0KCkgKSB7IHRocm93IG5ldyBBcmd1bWVudEV4Y2VwdGlvbiggIkFuIGVtcHR5IGVudW1lcmFibGUgd2FzIHNwZWNpZmllZCIgKTsKICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgfQoKICAgICAgICAgICAgeWllbGQgcmV0dXJuIGVudW1lcmF0b3JzLlNlbGVjdCggdCA9PiB0Lkl0ZW0yLkN1cnJlbnQgKS5Ub0ltbXV0YWJsZUxpc3QoKTsKCiAgICAgICAgICAgIHdoaWxlICh0cnVlKSB7CiAgICAgICAgICAgICAgICB2YXIgaSA9IDA7CiAgICAgICAgICAgICAgICB3aGlsZSh0cnVlKSB7CiAgICAgICAgICAgICAgICAgICAgaWYgKCBpID09IGVudW1lcmF0b3JzLkNvdW50ICkgewogICAgICAgICAgICAgICAgICAgICAgICB5aWVsZCBicmVhazsKICAgICAgICAgICAgICAgICAgICB9CgogICAgICAgICAgICAgICAgICAgIGlmICggIWVudW1lcmF0b3JzW2ldLkl0ZW0yLk1vdmVOZXh0KCkgKSB7CiAgICAgICAgICAgICAgICAgICAgICAgIGVudW1lcmF0b3JzW2ldID0gVHVwbGUuQ3JlYXRlKCBlbnVtZXJhdG9yc1tpXS5JdGVtMSwKICAgICAgICAgICAgICAgICAgICAgICAgICAgIGVudW1lcmF0b3JzW2ldLkl0ZW0xLkdldEVudW1lcmF0b3IoKSApOwogICAgICAgICAgICAgICAgICAgICAgICBlbnVtZXJhdG9yc1tpXS5JdGVtMi5Nb3ZlTmV4dCgpOwogICAgICAgICAgICAgICAgICAgIH0gZWxzZSB7CiAgICAgICAgICAgICAgICAgICAgICAgIGJyZWFrOwogICAgICAgICAgICAgICAgICAgIH0KCiAgICAgICAgICAgICAgICAgICAgaSsrOwogICAgICAgICAgICAgICAgfQoKICAgICAgICAgICAgICAgIHlpZWxkIHJldHVybiBlbnVtZXJhdG9ycy5TZWxlY3QoIHQgPT4gdC5JdGVtMi5DdXJyZW50ICkuVG9JbW11dGFibGVMaXN0KCk7CiAgICAgICAgICAgIH0KICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgc3RyaW5nIENvbnZlcnRUb1R5cGVQYXJhbWV0ZXJzKEltbXV0YWJsZUFycmF5PElUeXBlUGFyYW1ldGVyU3ltYm9sPiBtZXRob2REZWNsYXJhdGlvblR5cGVQYXJhbWV0ZXJzKQogICAgICAgIHsKICAgICAgICAgICAgaWYgKG1ldGhvZERlY2xhcmF0aW9uVHlwZVBhcmFtZXRlcnMuTGVuZ3RoID09IDApCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHJldHVybiAiIjsKICAgICAgICAgICAgfQoKICAgICAgICAgICAgcmV0dXJuCiAgICAgICAgICAgICAgICAkIjx7c3RyaW5nLkpvaW4oIiwgIiwgbWV0aG9kRGVjbGFyYXRpb25UeXBlUGFyYW1ldGVycy5TZWxlY3QodHlwZVBhcmFtZXRlciA9PiB0eXBlUGFyYW1ldGVyLk5hbWUpKX0+IjsKICAgICAgICB9CiAgICB9Cn0="
    },
    "nloum/CodeIO@a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631:FluentSourceGenerators/Utilities.cs": {
      "sha256": "311025d129bb7494926d076e98ddc182ba057f9a177e36d44a2af2437bf3f906",
      "base64": "77u/dXNpbmcgU3lzdGVtOwp1c2luZyBTeXN0ZW0uQ29sbGVjdGlvbnMuR2VuZXJpYzsKdXNpbmcgU3lzdGVtLkNvbGxlY3Rpb25zLkltbXV0YWJsZTsKdXNpbmcgU3lzdGVtLkxpbnE7CnVzaW5nIEh1bWFuaXplcjsKdXNpbmcgTWljcm9zb2Z0LkNvZGVBbmFseXNpczsKdXNpbmcgTWljcm9zb2Z0LkNvZGVBbmFseXNpcy5DU2hhcnA7CnVzaW5nIE1pY3Jvc29mdC5Db2RlQW5hbHlzaXMuQ1NoYXJwLlN5bnRheDsKCm5hbWVzcGFjZSBGbHVlbnRTb3VyY2VHZW5lcmF0b3JzCnsKICAgIHB1YmxpYyBjbGFzcyBVdGlsaXRpZXMKICAgIHsKICAgICAgICBwdWJsaWMgc3RhdGljIFN5bnRheE5vZGUgRmluZFBhcmVudE9mS2luZChTeW50YXhOb2RlIGNoaWxkLCBTeW50YXhLaW5kIHN5bnRheEtpbmQpCiAgICAgICAgewogICAgICAgICAgICBpZiAoY2hpbGQuSXNLaW5kKHN5bnRheEtpbmQpKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICByZXR1cm4gY2hpbGQ7CiAgICAgICAgICAgIH0KCiAgICAgICAgICAgIGlmIChjaGlsZC5QYXJlbnQgIT0gbnVsbCkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgcmV0dXJuIEZpbmRQYXJlbnRPZktpbmQoY2hpbGQuUGFyZW50LCBzeW50YXhLaW5kKTsKICAgICAgICAgICAgfQoKICAgICAgICAgICAgcmV0dXJuIG51bGw7CiAgICAgICAgfQoKICAgICAgICBwdWJsaWMgc3RhdGljIEltbXV0YWJsZUxpc3Q8VD4gR2V0RGVzY2VuZGFudHNPZlR5cGU8VD4oU3ludGF4Tm9kZSBub2RlKSB3aGVyZSBUIDogU3ludGF4Tm9kZQogICAgICAgIHsKICAgICAgICAgICAgdmFyIHJlc3VsdCA9IG5ldyBMaXN0PFQ+KCk7CiAgICAgICAgICAgIFRyYXZlcnNlVHJlZShub2RlLCBkZXNjZW5kYW50ID0+CiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIGlmIChkZXNjZW5kYW50IGlzIFQgdCkKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICByZXN1bHQuQWRkKHQpOwogICAgICAgICAgICAgICAgfQogICAgICAgICAgICB9KTsKICAgICAgICAgICAgcmV0dXJuIHJlc3VsdC5Ub0ltbXV0YWJsZUxpc3QoKTsKICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgSW1tdXRhYmxlTGlzdDxTeW50YXhOb2RlPiBHZXREZXNjZW5kYW50cyhTeW50YXhOb2RlIG5vZGUpCiAgICAgICAgewogICAgICAgICAgICB2YXIgcmVzdWx0ID0gbmV3IExpc3Q8U3ludGF4Tm9kZT4oKTsKICAgICAgICAgICAgVHJhdmVyc2VUcmVlKG5vZGUsIGRlc2NlbmRhbnQgPT4KICAgICAgICAgICAgewogICAgICAgICAgICAgICAgcmVzdWx0LkFkZChkZXNjZW5kYW50KTsKICAgICAgICAgICAgfSk7CiAgICAgICAgICAgIHJldHVybiByZXN1bHQuVG9JbW11dGFibGVMaXN0KCk7CiAgICAgICAgfQoKICAgICAgICBwdWJsaWMgc3RhdGljIHZvaWQgVHJhdmVyc2VUcmVlKFN5bnRheE5vZGUgc3ludGF4Tm9kZSwgQWN0aW9uPFN5bnRheE5vZGU+IHZpc2l0KQogICAgICAgIHsKICAgICAgICAgICAgdmlzaXQoc3ludGF4Tm9kZSk7CiAgICAgICAgICAgIGZvcmVhY2ggKHZhciBjaGlsZCBpbiBzeW50YXhOb2RlLkNoaWxkTm9kZXMoKSkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgVHJhdmVyc2VUcmVlKGNoaWxkLCB2aXNpdCk7CiAgICAgICAgICAgIH0KICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgc3RyaW5nIENvbnZlcnRUb1BhcmFtZXRlcihJUGFyYW1ldGVyU3ltYm9sIHBhcmFtZXRlcikKICAgICAgICB7CiAgICAgICAgICAgIHZhciBtb2RpZmllciA9IHN0cmluZy5Kb2luKCIgIiwgcGFyYW1ldGVyLkN1c3RvbU1vZGlmaWVycyk7CiAgICAgICAgICAgIGlmIChwYXJhbWV0ZXIuUmVmS2luZCA9PSBSZWZLaW5kLkluKQogICAgICAgICAgICAgICAgbW9kaWZpZXIgKz0gIiBpbiI7CiAgICAgICAgICAgIGlmIChwYXJhbWV0ZXIuUmVmS2luZCA9PSBSZWZLaW5kLk91dCkKICAgICAgICAgICAgICAgIG1vZGlmaWVyICs9ICIgb3V0IjsKICAgICAgICAgICAgaWYgKHBhcmFtZXRlci5SZWZLaW5kID09IFJlZktpbmQuUmVmKQogICAgICAgICAgICAgICAgbW9kaWZpZXIgKz0gIiByZWYiOwogICAgICAgICAgICByZXR1cm4gJCJ7bW9kaWZpZXJ9IHtwYXJhbWV0ZXIuVHlwZX0ge3BhcmFtZXRlci5OYW1lfSI7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHB1YmxpYyBzdGF0aWMgc3RyaW5nIENvbnZlcnRUb0FyZ3VtZW50KElQYXJhbWV0ZXJTeW1ib2wgcGFyYW1ldGVyKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIG1vZGlmaWVyID0gc3RyaW5nLkpvaW4oIiAiLCBwYXJhbWV0ZXIuQ3VzdG9tTW9kaWZpZXJzKTsKICAgICAgICAgICAgaWYgKHBhcmFtZXRlci5SZWZLaW5kID09IFJlZktpbmQuSW4pCiAgICAgICAgICAgICAgICBtb2RpZmllciArPSAiIGluIjsKICAgICAgICAgICAgaWYgKHBhcmFtZXRlci5SZWZLaW5kID09IFJlZktpbmQuT3V0KQogICAgICAgICAgICAgICAgbW9kaWZpZXIgKz0gIiBvdXQiOwogICAgICAgICAgICBpZiAocGFyYW1ldGVyLlJlZktpbmQgPT0gUmVmS2luZC5SZWYpCiAgICAgICAgICAgICAgICBtb2RpZmllciArPSAiIHJlZiI7CiAgICAgICAgICAgIHJldHVybiAkInttb2RpZmllcn0ge3BhcmFtZXRlci5OYW1lfSI7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHB1YmxpYyBzdGF0aWMgSUVudW1lcmFibGU8c3RyaW5nPiBHZXRVc2luZ3MoSVR5cGVTeW1ib2wgdHlwZVN5bWJvbCkKICAgICAgICB7CiAgICAgICAgICAgIHZhciByZXN1bHRzID0gbmV3IExpc3Q8c3RyaW5nPigpOwogICAgICAgICAgICBHZXRVc2luZ3ModHlwZVN5bWJvbCwgcmVzdWx0cyk7CiAgICAgICAgICAgIHJldHVybiByZXN1bHRzOwogICAgICAgIH0KCiAgICAgICAgcHJpdmF0ZSBzdGF0aWMgdm9pZCBHZXRVc2luZ3MoSVR5cGVTeW1ib2wgdHlwZVN5bWJvbCwgTGlzdDxzdHJpbmc+IHJlc3VsdHMpCiAgICAgICAgewogICAgICAgICAgICBpZiAodHlwZVN5bWJvbC5Db250YWluaW5nTmFtZXNwYWNlICE9IG51bGwpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHZhciByZXN1bHQgPSBzdHJpbmcuSm9pbigiLiIsIHR5cGVTeW1ib2wuQ29udGFpbmluZ05hbWVzcGFjZS5Db25zdGl0dWVudE5hbWVzcGFjZXMpOwogICAgICAgICAgICAgICAgaWYgKHJlc3VsdCAhPSAiPGdsb2JhbCBuYW1lc3BhY2U+IikKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICByZXN1bHRzLkFkZCgidXNpbmcgIiArIHN0cmluZy5Kb2luKCIuIiwgdHlwZVN5bWJvbC5Db250YWluaW5nTmFtZXNwYWNlLkNvbnN0aXR1ZW50TmFtZXNwYWNlcykgKyAiOyIpOwogICAgICAgICAgICAgICAgfQogICAgICAgICAgICB9CgogICAgICAgICAgICBpZiAodHlwZVN5bWJvbCBpcyBJTmFtZWRUeXBlU3ltYm9sIG5hbWVkVHlwZVN5bWJvbCkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgZm9yZWFjaCAodmFyIHR5cGVBcmd1bWVudCBpbiBuYW1lZFR5cGVTeW1ib2wuVHlwZUFyZ3VtZW50cykKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICBHZXRVc2luZ3ModHlwZUFyZ3VtZW50LCByZXN1bHRzKTsKICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgfQogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgc3RhdGljIEltbXV0YWJsZURpY3Rpb25hcnk8c3RyaW5nLCBJbW11dGFibGVMaXN0PElTeW1ib2w+PiBHZXRNZW1iZXJzR3JvdXBlZEJ5RGVjbGFyaW5nVHlwZShJbnRlcmZhY2VEZWNsYXJhdGlvblN5bnRheCBzeW50YXgsIFNlbWFudGljTW9kZWwgc2VtYW50aWNNb2RlbCkKICAgICAgICB7CiAgICAgICAgICAgIHZhciBzeW1ib2wgPSBNb2RlbEV4dGVuc2lvbnMuR2V0RGVjbGFyZWRTeW1ib2woc2VtYW50aWNNb2RlbCwgc3ludGF4KTsKICAgICAgICAgICAgcmV0dXJuIEdldE1lbWJlcnNHcm91cGVkQnlEZWNsYXJpbmdUeXBlKHN5bWJvbCBhcyBJTmFtZWRUeXBlU3ltYm9sKTsKICAgICAgICB9CgogICAgICAgIHB1YmxpYyBzdGF0aWMgSW1tdXRhYmxlRGljdGlvbmFyeTxzdHJpbmcsIEltbXV0YWJsZUxpc3Q8SVN5bWJvbD4+IEdldE1lbWJlcnNHcm91cGVkQnlEZWNsYXJpbmdUeXBlKElOYW1lZFR5cGVTeW1ib2wgc3ltYm9sKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIHJlc3VsdHMgPSBuZXcgRGljdGlvbmFyeTxzdHJpbmcsIEltbXV0YWJsZUxpc3Q8SVN5bWJvbD4+KCk7CiAgICAgICAgICAgIEdldE1lbWJlcnNHcm91cGVkQnlEZWNsYXJpbmdUeXBlKHN5bWJvbCwgcmVzdWx0cyk7CiAgICAgICAgICAgIHJldHVybiByZXN1bHRzLlRvSW1tdXRhYmxlRGljdGlvbmFyeSgpOwogICAgICAgIH0KICAgICAgICAKICAgICAgICBwcml2YXRlIHN0YXRpYyB2b2lkIEdldE1lbWJlcnNHcm91cGVkQnlEZWNsYXJpbmdUeXBlKElOYW1lZFR5cGVTeW1ib2wgc3ltYm9sLCBEaWN0aW9uYXJ5PHN0cmluZywgSW1tdXRhYmxlTGlzdDxJU3ltYm9sPj4gcmVzdWx0cykKICAgICAgICB7CiAgICAgICAgICAgIHZhciB0eXBlQXJndW1lbnRzID0gc3ltYm9sLlR5cGVBcmd1bWVudHMuTGVuZ3RoID09IDAgPyAiIiA6ICQiPHtzdHJpbmcuSm9pbigiLCAiLCBzeW1ib2wuVHlwZUFyZ3VtZW50cyl9PiI7CiAgICAgICAgICAgIHZhciBrZXkgPSAkIntzeW1ib2wuTmFtZX17dHlwZUFyZ3VtZW50c30iOwogICAgICAgICAgICBpZiAocmVzdWx0cy5Db250YWluc0tleShrZXkpKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICByZXR1cm47CiAgICAgICAgICAgIH0KCiAgICAgICAgICAgIHJlc3VsdHMuQWRkKGtleSwgc3ltYm9sLkdldE1lbWJlcnMoKS5Ub0ltbXV0YWJsZUxpc3QoKSk7CiAgICAgICAgICAgIAogICAgICAgICAgICBmb3JlYWNoICh2YXIgYmFzZUludGVyZmFjZSBpbiBzeW1ib2wuSW50ZXJmYWNlcykKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgR2V0TWVtYmVyc0dyb3VwZWRCeURlY2xhcmluZ1R5cGUoYmFzZUludGVyZmFjZSwgcmVzdWx0cyk7CiAgICAgICAgICAgIH0KICAgICAgICB9CiAgICAgICAgCiAgICAgICAgcHVibGljIHN0YXRpYyBib29sIElzQmFzZUNsYXNzKElOYW1lZFR5cGVTeW1ib2wgc3VwZXJDbGFzcywgSU5hbWVkVHlwZVN5bWJvbCBiYXNlQ2xhc3MpCiAgICAgICAgewogICAgICAgICAgICB2YXIgYmFzZUNsYXNzZXMgPSBuZXcgTGlzdDxJTmFtZWRUeXBlU3ltYm9sPigpOwogICAgICAgICAgICBHZXRCYXNlQ2xhc3NlcyhzdXBlckNsYXNzLCBiYXNlQ2xhc3Nlcyk7CiAgICAgICAgICAgIHJldHVybiBiYXNlQ2xhc3Nlcy5BbnkoeCA9PiB4LlRvU3RyaW5nKCkgPT0gYmFzZUNsYXNzLlRvU3RyaW5nKCkpOwogICAgICAgIH0KCiAgICAgICAgcHVibGljIHN0YXRpYyBzdHJpbmcgR2V0V2l0aG91dFR5cGVBcmd1bWVudHMoc3RyaW5nIHR5cGUpCiAgICAgICAgewogICAgICAgICAgICB2YXIgaWR4ID0gdHlwZS5JbmRleE9mKCc8Jyk7CiAgICAgICAgICAgIGlmIChpZHggPj0gMCkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgcmV0dXJuIHR5cGUuU3Vic3RyaW5nKDAsIGlkeCk7CiAgICAgICAgICAgIH0KCiAgICAgICAgICAgIHJldHVybiB0eXBlOwogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgc3RhdGljIElSZWFkT25seUxpc3Q8SU5hbWVkVHlwZVN5bWJvbD4gR2V0QmFzZUNsYXNzZXMoSU5hbWVkVHlwZVN5bWJvbCBzdXBlckNsYXNzKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIGJhc2VDbGFzc2VzID0gbmV3IExpc3Q8SU5hbWVkVHlwZVN5bWJvbD4oKTsKICAgICAgICAgICAgR2V0QmFzZUNsYXNzZXMoc3VwZXJDbGFzcywgYmFzZUNsYXNzZXMpOwogICAgICAgICAgICByZXR1cm4gYmFzZUNsYXNzZXM7CiAgICAgICAgfQogICAgICAgIAogICAgICAgIHByaXZhdGUgc3RhdGljIHZvaWQgR2V0QmFzZUNsYXNzZXMoSU5hbWVkVHlwZVN5bWJvbCBzdXBlckNsYXNzLCBMaXN0PElOYW1lZFR5cGVTeW1ib2w+IHJlc3VsdCkKICAgICAgICB7CiAgICAgICAgICAgIHJlc3VsdC5BZGQoc3VwZXJDbGFzcyk7CiAgICAgICAgICAgIGlmIChzdXBlckNsYXNzPy5CYXNlVHlwZT8uVHlwZUtpbmQgPT0gVHlwZUtpbmQuQ2xhc3MpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIEdldEJhc2VDbGFzc2VzKHN1cGVyQ2xhc3MuQmFzZVR5cGUsIHJlc3VsdCk7CiAgICAgICAgICAgIH0KICAgICAgICB9CiAgICAgICAgCiAgICAgICAgcHVibGljIHN0YXRpYyBib29sIElzQmFzZUludGVyZmFjZShJTmFtZWRUeXBlU3ltYm9sIHN1cGVySW50ZXJmYWNlLCBJTmFtZWRUeXBlU3ltYm9sIGJhc2VJbnRlcmZhY2UpCiAgICAgICAgewogICAgICAgICAgICB2YXIgYmFzZUludGVyZmFjZXMgPSBuZXcgTGlzdDxJTmFtZWRUeXBlU3ltYm9sPigpOwogICAgICAgICAgICBHZXRCYXNlSW50ZXJmYWNlcyhzdXBlckludGVyZmFjZSwgYmFzZUludGVyZmFjZXMpOwogICAgICAgICAgICByZXR1cm4gYmFzZUludGVyZmFjZXMuQW55KHggPT4geC5Ub1N0cmluZygpID09IGJhc2VJbnRlcmZhY2UuVG9TdHJpbmcoKSk7CiAgICAgICAgfQoKICAgICAgICBwdWJsaWMgc3RhdGljIElSZWFkT25seUxpc3Q8SU5hbWVkVHlwZVN5bWJvbD4gR2V0QmFzZUludGVyZmFjZXMoSU5hbWVkVHlwZVN5bWJvbCBzdXBlckludGVyZmFjZSkKICAgICAgICB7CiAgICAgICAgICAgIHZhciBiYXNlSW50ZXJmYWNlcyA9IG5ldyBMaXN0PElOYW1lZFR5cGVTeW1ib2w+KCk7CiAgICAgICAgICAgIEdldEJhc2VJbnRlcmZhY2VzKHN1cGVySW50ZXJmYWNlLCBiYXNlSW50ZXJmYWNlcyk7CiAgICAgICAgICAgIHJldHVybiBiYXNlSW50ZXJmYWNlczsKICAgICAgICB9CiAgICAgICAgCiAgICAgICAgcHJpdmF0ZSBzdGF0aWMgdm9pZCBHZXRCYXNlSW50ZXJmYWNlcyhJTmFtZWRUeXBlU3ltYm9sIHN1cGVySW50ZXJmYWNlLCBMaXN0PElOYW1lZFR5cGVTeW1ib2w+IHJlc3VsdCkKICAgICAgICB7CiAgICAgICAgICAgIHJlc3VsdC5BZGQoc3VwZXJJbnRlcmZhY2UpOwogICAgICAgICAgICBmb3JlYWNoICh2YXIgYmFzZVR5cGUgaW4gc3VwZXJJbnRlcmZhY2UuSW50ZXJmYWNlcykKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgR2V0QmFzZUludGVyZmFjZXMoYmFzZVR5cGUsIHJlc3VsdCk7CiAgICAgICAgICAgIH0KICAgICAgICB9CiAgICAgICAgCiAgICAgICAgcHVibGljIHN0YXRpYyBzdHJpbmcgR2VuZXJhdGVWYXJpYWJsZU5hbWUoc3RyaW5nIHR5cGVOYW1lLCBib29sIGlzSW50ZXJmYWNlKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIHJlc3VsdCA9IGlzSW50ZXJmYWNlID8gdHlwZU5hbWUuU3Vic3RyaW5nKDEpIDogdHlwZU5hbWU7CiAgICAgICAgICAgIGlmIChyZXN1bHQuQ29udGFpbnMoIjwiKSkKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgcmVzdWx0ID0gcmVzdWx0LlN1YnN0cmluZygwLCByZXN1bHQuSW5kZXhPZignPCcpKTsKICAgICAgICAgICAgfQoKICAgICAgICAgICAgcmV0dXJuIHJlc3VsdC5DYW1lbGl6ZSgpOwogICAgICAgIH0KICAgICAgICAKICAgICAgICAvLy8gPHN1bW1hcnk+CiAgICAgICAgLy8vIElmIHlvdSBwYXNzIGluIG5ld1tdezAsIDF9IGFuZCBuZXdbXXswLCAxfSB5b3UnbGwgZ2V0IG5ld1tdezAsIDB9LCBuZXdbXXswLCAxfSwgbmV3W117MSwgMH0sIG5ld1tdezEsIDF9IG91dC4KICAgICAgICAvLy8gPC9zdW1tYXJ5PgogICAgICAgIC8vLyA8dHlwZXBhcmFtIG5hbWU9IlRJdGVtIj5UaGUgdHlwZSBvZiBpdGVtIGluIHRoZSBlbnVtZXJhYmxlcy48L3R5cGVwYXJhbT4KICAgICAgICAvLy8gPHBhcmFtIG5hbWU9ImVudW1lcmFibGVzIj5UaGUgZW51bWVyYWJsZXMgdGhhdCB3aWxsIGJlIHVzZWQgdG8gY2FsY3VsYXRlIGNvbWJpbmF0aW9uczwvcGFyYW0+CiAgICAgICAgLy8vIDxyZXR1cm5zPkEgbGF6aWx5LWNvbXB1dGVkIGVudW1lcmFibGUgd2hlcmUgZWFjaCByZXR1cm4gaXRlbSBpcyBhIGNvbWJpbmF0aW9uPC9yZXR1cm5zPgogICAgICAgIHB1YmxpYyBzdGF0aWMgSUVudW1lcmFibGU8SW1tdXRhYmxlTGlzdDxUSXRlbT4+IENhbGNDb21iaW5hdGlvbnNPZk9uZUZyb21FYWNoPFRJdGVtPigKICAgICAgICAgICAgSUVudW1lcmFibGU8SUVudW1lcmFibGU8VEl0ZW0+PiBlbnVtZXJhYmxlcyApIHsKICAgICAgICAgICAgdmFyIGVudW1lcmF0b3JzID0gZW51bWVyYWJsZXMuU2VsZWN0KCBlbnVtZXJhYmxlID0+IFR1cGxlLkNyZWF0ZSggZW51bWVyYWJsZSwgZW51bWVyYWJsZS5HZXRFbnVtZXJhdG9yKCkgKSApLlRvTGlzdCgpOwoKICAgICAgICAgICAgZm9yZWFjaCAoIHZhciBlbnVtZXJhdG9yIGluIGVudW1lcmF0b3JzICkgewogICAgICAgICAgICAgICAgaWYgKCAhZW51bWVyYXRvci5JdGVtMi5Nb3ZlTmV4dCgpICkgeyB0aHJvdyBuZXcgQXJndW1lbnRFeGNlcHRpb24oICJBbiBlbXB0eSBlbnVtZXJhYmxlIHdhcyBzcGVjaWZpZWQiICk7CiAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgIH0KCiAgICAgICAgICAgIHlpZWxkIHJldHVybiBlbnVtZXJhdG9ycy5TZWxlY3QoIHQgPT4gdC5JdGVtMi5DdXJyZW50ICkuVG9JbW11dGFibGVMaXN0KCk7CgogICAgICAgICAgICB3aGlsZSAodHJ1ZSkgewogICAgICAgICAgICAgICAgdmFyIGkgPSAwOwogICAgICAgICAgICAgICAgd2hpbGUodHJ1ZSkgewogICAgICAgICAgICAgICAgICAgIGlmICggaSA9PSBlbnVtZXJhdG9ycy5Db3VudCApIHsKICAgICAgICAgICAgICAgICAgICAgICAgeWllbGQgYnJlYWs7CiAgICAgICAgICAgICAgICAgICAgfQoKICAgICAgICAgICAgICAgICAgICBpZiAoICFlbnVtZXJhdG9yc1tpXS5JdGVtMi5Nb3ZlTmV4dCgpICkgewogICAgICAgICAgICAgICAgICAgICAgICBlbnVtZXJhdG9yc1tpXSA9IFR1cGxlLkNyZWF0ZSggZW51bWVyYXRvcnNbaV0uSXRlbTEsCiAgICAgICAgICAgICAgICAgICAgICAgICAgICBlbnVtZXJhdG9yc1tpXS5JdGVtMS5HZXRFbnVtZXJhdG9yKCkgKTsKICAgICAgICAgICAgICAgICAgICAgICAgZW51bWVyYXRvcnNbaV0uSXRlbTIuTW92ZU5leHQoKTsKICAgICAgICAgICAgICAgICAgICB9IGVsc2UgewogICAgICAgICAgICAgICAgICAgICAgICBicmVhazsKICAgICAgICAgICAgICAgICAgICB9CgogICAgICAgICAgICAgICAgICAgIGkrKzsKICAgICAgICAgICAgICAgIH0KCiAgICAgICAgICAgICAgICB5aWVsZCByZXR1cm4gZW51bWVyYXRvcnMuU2VsZWN0KCB0ID0+IHQuSXRlbTIuQ3VycmVudCApLlRvSW1tdXRhYmxlTGlzdCgpOwogICAgICAgICAgICB9CiAgICAgICAgfQoKICAgICAgICBwdWJsaWMgc3RhdGljIHN0cmluZyBDb252ZXJ0VG9UeXBlUGFyYW1ldGVycyhJbW11dGFibGVBcnJheTxJVHlwZVBhcmFtZXRlclN5bWJvbD4gbWV0aG9kRGVjbGFyYXRpb25UeXBlUGFyYW1ldGVycykKICAgICAgICB7CiAgICAgICAgICAgIGlmIChtZXRob2REZWNsYXJhdGlvblR5cGVQYXJhbWV0ZXJzLkxlbmd0aCA9PSAwKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICByZXR1cm4gIiI7CiAgICAgICAgICAgIH0KCiAgICAgICAgICAgIHJldHVybgogICAgICAgICAgICAgICAgJCI8e3N0cmluZy5Kb2luKCIsICIsIG1ldGhvZERlY2xhcmF0aW9uVHlwZVBhcmFtZXRlcnMuU2VsZWN0KHR5cGVQYXJhbWV0ZXIgPT4gdHlwZVBhcmFtZXRlci5OYW1lKSl9PiI7CiAgICAgICAgfQogICAgfQp9"
    },
    "nloum/letter-classifier@c97dd60d4a5061e5086f7c7e3db74ad6593d0afe:MainViewModel.cs": {
      "sha256": "67a54442b05f8c8f726e7ca03b228def8e47b7a5d9762ee84bfa43ca45132d49",
      "base64": "77u/dXNpbmcgQ3N2SGVscGVyOw0KdXNpbmcgTWljcm9zb2Z0LldpbjMyOw0KdXNpbmcgUmVhY3RpdmUuQmluZGluZ3M7DQp1c2luZyBTeXN0ZW07DQp1c2luZyBTeXN0ZW0uQ29sbGVjdGlvbnMuR2VuZXJpYzsNCnVzaW5nIFN5c3RlbS5Db21wb25lbnRNb2RlbDsNCnVzaW5nIFN5c3RlbS5JTzsNCnVzaW5nIFN5c3RlbS5MaW5xOw0KdXNpbmcgU3lzdGVtLlJlYWN0aXZlLkxpbnE7DQp1c2luZyBTeXN0ZW0uVGV4dDsNCnVzaW5nIFN5c3RlbS5UZXh0LlJlZ3VsYXJFeHByZXNzaW9uczsNCnVzaW5nIFN5c3RlbS5UaHJlYWRpbmcuVGFza3M7DQoNCm5hbWVzcGFjZSBMZXR0ZXJDbGFzc2lmaWVyDQp7DQogICAgY2xhc3MgTWFpblZpZXdNb2RlbA0KICAgIHsNCiAgICAgICAgcHJpdmF0ZSBzdHJpbmcgX2ZvbGRlclBhdGggPSBAIlxcTWFjXEhvbWVcRG9jdW1lbnRzXHN0YXQtNjAyLTIwMTktc3ByaW5nXHBsb3RzIjsNCg0KICAgICAgICBwcml2YXRlIExpc3Q8TWFudWFsQ2xhc3NpZmljYXRpb24+IG1hbnVhbENsYXNzaWZpY2F0aW9ucyA9IG5ldyBMaXN0PE1hbnVhbENsYXNzaWZpY2F0aW9uPigpOw0KICAgICAgICBwcml2YXRlIExpc3Q8aW50PiB4VmFsdWVzID0gbmV3IExpc3Q8aW50PigpOw0KDQogICAgICAgIHB1YmxpYyBNYWluVmlld01vZGVsKCkNCiAgICAgICAgew0KICAgICAgICAgICAgU2F2ZUNsYXNzaWZpY2F0aW9uc0NvbW1hbmQuU3Vic2NyaWJlKF8gPT4gU2F2ZUNsYXNzaWZpY2F0aW9ucygpKTsNCiAgICAgICAgICAgIExvYWRDbGFzc2lmaWNhdGlvbnNDb21tYW5kLlN1YnNjcmliZShfID0+IExvYWRDbGFzc2lmaWNhdGlvbnMoKSk7DQogICAgICAgICAgICBMYWJlbEltYWdlQ29tbWFuZCA9IG5ldyBSZWFjdGl2ZUNvbW1hbmQoTGV0dGVyLlNlbGVjdCh4ID0+ICFzdHJpbmcuSXNOdWxsT3JFbXB0eSh4KSkpOw0KICAgICAgICAgICAgTGFiZWxJbWFnZUNvbW1hbmQuU3Vic2NyaWJlKF8gPT4gTGFiZWxJbWFnZSgpKTsNCiAgICAgICAgICAgIExvYWRGb2xkZXIoKTsNCiAgICAgICAgfQ0KDQogICAgICAgIHByaXZhdGUgdm9pZCBMb2FkRm9sZGVyKCkgew0KICAgICAgICAgICAgeFZhbHVlcy5DbGVhcigpOw0KICAgICAgICAgICAgdmFyIGltYWdlRmlsZXMgPSBEaXJlY3RvcnkuRW51bWVyYXRlRmlsZXMoX2ZvbGRlclBhdGgsICIqLnBuZyIpOw0KICAgICAgICAgICAgZm9yZWFjaCh2YXIgaW1hZ2VGaWxlIGluIGltYWdlRmlsZXMpDQogICAgICAgICAgICB7DQogICAgICAgICAgICAgICAgdmFyIG1hdGNoID0gX2ltYWdlRmlsZU5hbWVSZWdleC5NYXRjaChQYXRoLkdldEZpbGVOYW1lKGltYWdlRmlsZSkpOw0KICAgICAgICAgICAgICAgIGlmICghbWF0Y2guU3VjY2VzcykNCiAgICAgICAgICAgICAgICB7DQogICAgICAgICAgICAgICAgICAgIGNvbnRpbnVlOw0KICAgICAgICAgICAgICAgIH0NCiAgICAgICAgICAgICAgICB4VmFsdWVzLkFkZChpbnQuUGFyc2UobWF0Y2guR3JvdXBzWzFdLlZhbHVlKSk7DQogICAgICAgICAgICB9DQogICAgICAgICAgICBQcmVwYXJlVG9DbGFzc2lmeU5leHRJbWFnZSgpOw0KICAgICAgICB9DQoNCiAgICAgICAgcHJpdmF0ZSB2b2lkIFByZXBhcmVUb0NsYXNzaWZ5TmV4dEltYWdlKCkNCiAgICAgICAgew0KICAgICAgICAgICAgZm9yZWFjaCh2YXIgeCBpbiB4VmFsdWVzKQ0KICAgICAgICAgICAgew0KICAgICAgICAgICAgICAgIGlmICghbWFudWFsQ2xhc3NpZmljYXRpb25zLkFueShjID0+IGMuWCA9PSB4KSkNCiAgICAgICAgICAgICAgICB7DQogICAgICAgICAgICAgICAgICAgIEltYWdlUGF0aC5WYWx1ZSA9IF9mb2xkZXJQYXRoICsgIlxcaW1hZ2UiICsgeCArICIucG5nIjsNCiAgICAgICAgICAgICAgICAgICAgTGV0dGVyLlZhbHVlID0gIiI7DQogICAgICAgICAgICAgICAgICAgIGJyZWFrOw0KICAgICAgICAgICAgICAgIH0NCiAgICAgICAgICAgIH0NCiAgICAgICAgfQ0KDQogICAgICAgIHByaXZhdGUgdm9pZCBMb2FkQ2xhc3NpZmljYXRpb25zKCkNCiAgICAgICAgew0KICAgICAgICAgICAgdmFyIGZpbGVPcGVuRGxnID0gbmV3IE9wZW5GaWxlRGlhbG9nKCk7DQogICAgICAgICAgICBmaWxlT3BlbkRsZy5EZWZhdWx0RXh0ID0gIiouY3N2IjsNCiAgICAgICAgICAgIGlmIChmaWxlT3BlbkRsZy5TaG93RGlhbG9nKCkgPT0gdHJ1ZSkNCiAgICAgICAgICAgIHsNCiAgICAgICAgICAgICAgICB1c2luZyAodmFyIHN0cmVhbVJlYWRlciA9IG5ldyBTdHJlYW1SZWFkZXIoZmlsZU9wZW5EbGcuRmlsZU5hbWUpKQ0KICAgICAgICAgICAgICAgIHVzaW5nICh2YXIgcmVhZGVyID0gbmV3IENzdlJlYWRlcihzdHJlYW1SZWFkZXIpKQ0KICAgICAgICAgICAgICAgIHsNCiAgICAgICAgICAgICAgICAgICAgbWFudWFsQ2xhc3NpZmljYXRpb25zID0gcmVhZGVyLkdldFJlY29yZHM8TWFudWFsQ2xhc3NpZmljYXRpb24+KCkuVG9MaXN0KCk7DQogICAgICAgICAgICAgICAgfQ0KICAgICAgICAgICAgfQ0KICAgICAgICAgICAgUHJlcGFyZVRvQ2xhc3NpZnlOZXh0SW1hZ2UoKTsNCiAgICAgICAgfQ0KDQogICAgICAgIHByaXZhdGUgdm9pZCBTYXZlQ2xhc3NpZmljYXRpb25zKCkNCiAgICAgICAgew0KICAgICAgICAgICAgdmFyIGZpbGVTYXZlRGxnID0gbmV3IFNhdmVGaWxlRGlhbG9nKCk7DQogICAgICAgICAgICBmaWxlU2F2ZURsZy5EZWZhdWx0RXh0ID0gIiouY3N2IjsNCiAgICAgICAgICAgIGlmIChmaWxlU2F2ZURsZy5TaG93RGlhbG9nKCkgPT0gdHJ1ZSkNCiAgICAgICAgICAgIHsNCiAgICAgICAgICAgICAgICB1c2luZyAodmFyIHN0cmVhbVdyaXRlciA9IG5ldyBTdHJlYW1Xcml0ZXIoZmlsZVNhdmVEbGcuRmlsZU5hbWUpKQ0KICAgICAgICAgICAgICAgIHVzaW5nICh2YXIgd3JpdGVyID0gbmV3IENzdldyaXRlcihzdHJlYW1Xcml0ZXIpKQ0KICAgICAgICAgICAgICAgIHsNCiAgICAgICAgICAgICAgICAgICAgd3JpdGVyLldyaXRlSGVhZGVyPE1hbnVhbENsYXNzaWZpY2F0aW9uPigpOw0KICAgICAgICAgICAgICAgICAgICB3cml0ZXIuV3JpdGVSZWNvcmRzKG1hbnVhbENsYXNzaWZpY2F0aW9ucyk7DQogICAgICAgICAgICAgICAgfQ0KICAgICAgICAgICAgfQ0KICAgICAgICB9DQoNCiAgICAgICAgcHJpdmF0ZSBzdGF0aWMgUmVnZXggX2ltYWdlRmlsZU5hbWVSZWdleCA9IG5ldyBSZWdleChAImltYWdlKFxkKylcLnBuZyIpOw0KDQogICAgICAgIHByaXZhdGUgdm9pZCBMYWJlbEltYWdlKCkNCiAgICAgICAgew0KICAgICAgICAgICAgdmFyIG1hdGNoID0gX2ltYWdlRmlsZU5hbWVSZWdleC5NYXRjaChQYXRoLkdldEZpbGVOYW1lKEltYWdlUGF0aC5WYWx1ZSkpOw0KICAgICAgICAgICAgdmFyIGNhcHR1cmUgPSBpbnQuUGFyc2UobWF0Y2guR3JvdXBzWzFdLlZhbHVlKTsNCg0KICAgICAgICAgICAgbWFudWFsQ2xhc3NpZmljYXRpb25zLkFkZChuZXcgTWFudWFsQ2xhc3NpZmljYXRpb24NCiAgICAgICAgICAgIHsNCiAgICAgICAgICAgICAgICBMZXR0ZXIgPSBMZXR0ZXIuVmFsdWUsDQogICAgICAgICAgICAgICAgWCA9IGNhcHR1cmUNCiAgICAgICAgICAgIH0pOw0KDQogICAgICAgICAgICBQcmVwYXJlVG9DbGFzc2lmeU5leHRJbWFnZSgpOw0KICAgICAgICB9DQoNCiAgICAgICAgcHVibGljIFJlYWN0aXZlQ29tbWFuZCBMYWJlbEltYWdlQ29tbWFuZCB7IGdldDsgfQ0KICAgICAgICBwdWJsaWMgUmVhY3RpdmVDb21tYW5kIFNhdmVDbGFzc2lmaWNhdGlvbnNDb21tYW5kIHsgZ2V0OyB9ID0gbmV3IFJlYWN0aXZlQ29tbWFuZCgpOw0KICAgICAgICBwdWJsaWMgUmVhY3RpdmVDb21tYW5kIExvYWRDbGFzc2lmaWNhdGlvbnNDb21tYW5kIHsgZ2V0OyB9ID0gbmV3IFJlYWN0aXZlQ29tbWFuZCgpOw0KDQogICAgICAgIHB1YmxpYyBSZWFjdGl2ZVByb3BlcnR5PHN0cmluZz4gSW1hZ2VQYXRoIHsgZ2V0OyB9ID0gbmV3IFJlYWN0aXZlUHJvcGVydHk8c3RyaW5nPigpOw0KICAgICAgICBwdWJsaWMgUmVhY3RpdmVQcm9wZXJ0eTxzdHJpbmc+IExldHRlciB7IGdldDsgfSA9IG5ldyBSZWFjdGl2ZVByb3BlcnR5PHN0cmluZz4oKTsNCiAgICB9DQp9DQo="
    },
    "nloum/PackageProjectDependencySwitcher@9fa4a1a917f491228f85c3b772ba8856bc5af7c0:src/PackageProjectDependencySwitcher/DependencySwitcherService.cs": {
      "sha256": "1c9193e9c2a2c5789fe1c22996f44d05e7e13eaecd23e383eaa8c66c291201e1",
      "base64": "77u/dXNpbmcgU3lzdGVtOwp1c2luZyBTeXN0ZW0uQ29sbGVjdGlvbnMuR2VuZXJpYzsKdXNpbmcgU3lzdGVtLkNvbGxlY3Rpb25zLkltbXV0YWJsZTsKdXNpbmcgU3lzdGVtLklPOwp1c2luZyBTeXN0ZW0uVGV4dC5SZWd1bGFyRXhwcmVzc2lvbnM7CnVzaW5nIElvRmx1ZW50bHk7CgpuYW1lc3BhY2UgUGFja2FnZVByb2plY3REZXBlbmRlbmN5U3dpdGNoZXIKewogICAgcHVibGljIGNsYXNzIERlcGVuZGVuY3lTd2l0Y2hlclNlcnZpY2UgOiBJRGVwZW5kZW5jeVN3aXRjaGVyU2VydmljZQogICAgewogICAgICAgIHByaXZhdGUgc3RhdGljIFJlZ2V4IF9wcm9qZWN0UmVmZXJlbmNlID0gbmV3IFJlZ2V4KEAiPFByb2plY3RSZWZlcmVuY2UgSW5jbHVkZT0iIig/PHByb2plY3RGb2xkZXJQYXRoPi4rKSg/PHByb2plY3RGaWxlTmFtZT5bXjxeXF4vXSspXC4oPzxwcm9qZWN0RmlsZUV4dGVuc2lvbj4uKykiIiIpOwogICAgICAgIHByaXZhdGUgc3RhdGljIFJlZ2V4IF9wYWNrYWdlVmVyc2lvbiA9IG5ldyBSZWdleChAIjxQYWNrYWdlVmVyc2lvbj4oPzxwYWNrYWdlVmVyc2lvbj5bXjxdKyk8L1BhY2thZ2VWZXJzaW9uPiIpOwoKICAgICAgICBwdWJsaWMgdm9pZCBDb252ZXJ0VG9Qcm9qZWN0UmVmZXJlbmNlcyhBYnNvbHV0ZVBhdGggc29sdXRpb25Gb2xkZXIsIEZ1bmM8QWJzb2x1dGVQYXRoLCBzdHJpbmcsIGJvb2w+IHJlZmVyZW5jZVByZWRpY2F0ZSwgYm9vbCBiYWNrdXApCiAgICAgICAgewogICAgICAgICAgICB2YXIgZmlsZXMgPSBzb2x1dGlvbkZvbGRlci5HZXREZXNjZW5kYW50cygiKi5jc3Byb2oiKS5Ub0ltbXV0YWJsZUxpc3QoKTsKCiAgICAgICAgICAgIHZhciBmaWxlc1RvUGFja2FnZXMgPSBuZXcgRGljdGlvbmFyeTxBYnNvbHV0ZVBhdGgsIEZ1bmM8c3RyaW5nLCBBYnNvbHV0ZVBhdGgsIHN0cmluZz4+KCk7CiAgICAgICAgICAgIAogICAgICAgICAgICBmb3JlYWNoICh2YXIgZmlsZSBpbiBmaWxlcykKICAgICAgICAgICAgewogICAgICAgICAgICAgICAgdmFyIHRleHQgPSBmaWxlLlJlYWRBbGxUZXh0KCk7CiAgICAgICAgICAgICAgICB2YXIgcGFja2FnZVZlcnNpb24gPSBfcGFja2FnZVZlcnNpb24uTWF0Y2godGV4dCk7CgogICAgICAgICAgICAgICAgaWYgKHBhY2thZ2VWZXJzaW9uLlN1Y2Nlc3MpCiAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgdmFyIHJlZ2V4ID0gbmV3IFJlZ2V4KCQiPFBhY2thZ2VSZWZlcmVuY2UgSW5jbHVkZT1cIntQYXRoLkdldEZpbGVOYW1lV2l0aG91dEV4dGVuc2lvbihmaWxlLk5hbWUpfVwiLisvPiIpOwoKICAgICAgICAgICAgICAgICAgICBmaWxlc1RvUGFja2FnZXNbZmlsZV0gPSAoY3Nwcm9qRmlsZVRleHQsIHJlbGF0aXZlVG8pID0+CiAgICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICAgICB2YXIgcGF0aCA9IGZpbGUuUmVsYXRpdmVUbyhyZWxhdGl2ZVRvKTsKCiAgICAgICAgICAgICAgICAgICAgICAgIHZhciByZXN1bHQgPSByZWdleC5SZXBsYWNlKGNzcHJvakZpbGVUZXh0LAogICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICQiPFByb2plY3RSZWZlcmVuY2UgSW5jbHVkZT1cIntwYXRofVwiIC8+Iik7CiAgICAgICAgICAgICAgICAgICAgICAgIHJldHVybiByZXN1bHQ7CiAgICAgICAgICAgICAgICAgICAgfTsKICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgfQogICAgICAgICAgICAKICAgICAgICAgICAgZm9yZWFjaCAodmFyIGZpbGUgaW4gZmlsZXMpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHZhciB0ZXh0ID0gZmlsZS5SZWFkQWxsVGV4dCgpOwoKICAgICAgICAgICAgICAgIHZhciBudW1DaGFuZ2VzID0gMDsKICAgICAgICAgICAgICAgIGZvcmVhY2ggKHZhciBwYWNrYWdlIGluIGZpbGVzVG9QYWNrYWdlcykKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICBpZiAocGFja2FnZS5LZXkuRXF1YWxzKGZpbGUpKQogICAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAgICAgY29udGludWU7CiAgICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgICAgIAogICAgICAgICAgICAgICAgICAgIHZhciBwcmV2VGV4dCA9IHRleHQ7CiAgICAgICAgICAgICAgICAgICAgdGV4dCA9IHBhY2thZ2UuVmFsdWUodGV4dCwgZmlsZS5QYXJlbnQoKSk7CiAgICAgICAgICAgICAgICAgICAgaWYgKCF0ZXh0LkVxdWFscyhwcmV2VGV4dCkpCiAgICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICAgICBudW1DaGFuZ2VzKys7CiAgICAgICAgICAgICAgICAgICAgICAgIENvbnNvbGUuV3JpdGVMaW5lKCQiQ2hhbmdpbmcge1BhdGguR2V0RmlsZU5hbWVXaXRob3V0RXh0ZW5zaW9uKGZpbGUuVG9TdHJpbmcoKSl9J3MgZGVwZW5kZW5jeSBvbiB7UGF0aC5HZXRGaWxlTmFtZVdpdGhvdXRFeHRlbnNpb24ocGFja2FnZS5LZXkuVG9TdHJpbmcoKSl9IGZyb20gYSBwYWNrYWdlIHJlZmVyZW5jZSB0byBhIHByb2plY3QgcmVmZXJlbmNlIik7CiAgICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgfQoKICAgICAgICAgICAgICAgIGlmIChudW1DaGFuZ2VzID4gMCkKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICBmaWxlLldyaXRlQWxsVGV4dCh0ZXh0KTsKICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgfQogICAgICAgIH0KICAgICAgICAKICAgICAgICBwdWJsaWMgdm9pZCBDb252ZXJ0VG9QYWNrYWdlUmVmZXJlbmNlcyhBYnNvbHV0ZVBhdGggc29sdXRpb25Gb2xkZXIsIEZ1bmM8QWJzb2x1dGVQYXRoLCBzdHJpbmcsIGJvb2w+IHJlZmVyZW5jZVByZWRpY2F0ZSwgYm9vbCBiYWNrdXApCiAgICAgICAgewogICAgICAgICAgICB2YXIgZmlsZXMgPSBzb2x1dGlvbkZvbGRlci5HZXREZXNjZW5kYW50cygiKi5jc3Byb2oiKS5Ub0ltbXV0YWJsZUxpc3QoKTsKCiAgICAgICAgICAgIHZhciBmaWxlc1RvUGFja2FnZXMgPSBuZXcgRGljdGlvbmFyeTxBYnNvbHV0ZVBhdGgsIEZ1bmM8c3RyaW5nLCBzdHJpbmc+PigpOwogICAgICAgICAgICAKICAgICAgICAgICAgZm9yZWFjaCAodmFyIGZpbGUgaW4gZmlsZXMpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHZhciB0ZXh0ID0gZmlsZS5SZWFkQWxsVGV4dCgpOwogICAgICAgICAgICAgICAgdmFyIHBhY2thZ2VWZXJzaW9uID0gX3BhY2thZ2VWZXJzaW9uLk1hdGNoKHRleHQpOwoKICAgICAgICAgICAgICAgIGlmIChwYWNrYWdlVmVyc2lvbi5TdWNjZXNzKQogICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgIHZhciByZWdleCA9IG5ldyBSZWdleCgkIjxQcm9qZWN0UmVmZXJlbmNlIEluY2x1ZGU9XCIuK3tmaWxlLk5hbWV9XCIgKi8+Iik7CgogICAgICAgICAgICAgICAgICAgIHZhciBwYWNrYWdlVmVyc2lvbk51bWJlciA9IHBhY2thZ2VWZXJzaW9uLkdyb3Vwc1sicGFja2FnZVZlcnNpb24iXS5WYWx1ZTsKICAgICAgICAgICAgICAgICAgICBmaWxlc1RvUGFja2FnZXNbZmlsZV0gPSBzdHIgPT4gcmVnZXguUmVwbGFjZShzdHIsICQiPFBhY2thZ2VSZWZlcmVuY2UgSW5jbHVkZT1cIntmaWxlLldpdGhvdXRFeHRlbnNpb24oKS5OYW1lfVwiIFZlcnNpb249XCJ7cGFja2FnZVZlcnNpb25OdW1iZXJ9XCIgLz4iKTsKICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgfQoKICAgICAgICAgICAgZm9yZWFjaCAodmFyIGZpbGUgaW4gZmlsZXMpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHZhciB0ZXh0ID0gZmlsZS5SZWFkQWxsVGV4dCgpOwoKICAgICAgICAgICAgICAgIGlmIChmaWxlLldpdGhvdXRFeHRlbnNpb24oKS5IYXNFeHRlbnNpb24oIi5UZXN0IikpCiAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgLy8gVE9ETyAtIG1ha2UgdGhpcyBiZWhhdmlvciBjb25maWd1cmFibGUuCiAgICAgICAgICAgICAgICAgICAgY29udGludWU7CiAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICAKICAgICAgICAgICAgICAgIHZhciBudW1DaGFuZ2VzID0gMDsKICAgICAgICAgICAgICAgIGZvcmVhY2ggKHZhciBwYWNrYWdlIGluIGZpbGVzVG9QYWNrYWdlcykKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICB2YXIgcHJldlRleHQgPSB0ZXh0OwogICAgICAgICAgICAgICAgICAgIHRleHQgPSBwYWNrYWdlLlZhbHVlKHRleHQpOwogICAgICAgICAgICAgICAgICAgIGlmICghdGV4dC5FcXVhbHMocHJldlRleHQpKQogICAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAgICAgbnVtQ2hhbmdlcysrOwogICAgICAgICAgICAgICAgICAgICAgICBDb25zb2xlLldyaXRlTGluZSgkIkNoYW5naW5nIHtmaWxlLldpdGhvdXRFeHRlbnNpb24oKS5OYW1lfSdzIGRlcGVuZGVuY3kgb24ge3BhY2thZ2UuS2V5LldpdGhvdXRFeHRlbnNpb24oKS5OYW1lfSBmcm9tIGEgcHJvamVjdCByZWZlcmVuY2UgdG8gYSBwYWNrYWdlIHJlZmVyZW5jZSIpOwogICAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIAogICAgICAgICAgICAgICAgZmlsZS5Xcml0ZUFsbFRleHQodGV4dCk7CiAgICAgICAgICAgIH0KICAgICAgICB9CiAgICB9Cn0="
    },
    "nloum/PackageProjectDependencySwitcher@73ad92ac00f9dd8cdf62fc6704acc7e250741f5c:src/PackageProjectDependencySwitcher/DependencySwitcherService.cs": {
      "sha256": "cb3037e992bcbc34247cab6b815a1995de8e88ffd4d12aedb9542be07681abb8",
      "base64": "77u/dXNpbmcgU3lzdGVtOwp1c2luZyBTeXN0ZW0uQ29sbGVjdGlvbnMuR2VuZXJpYzsKdXNpbmcgU3lzdGVtLkNvbGxlY3Rpb25zLkltbXV0YWJsZTsKdXNpbmcgU3lzdGVtLklPOwp1c2luZyBTeXN0ZW0uVGV4dC5SZWd1bGFyRXhwcmVzc2lvbnM7CnVzaW5nIElvRmx1ZW50bHk7CgpuYW1lc3BhY2UgUGFja2FnZVByb2plY3REZXBlbmRlbmN5U3dpdGNoZXIKewogICAgcHVibGljIGNsYXNzIERlcGVuZGVuY3lTd2l0Y2hlclNlcnZpY2UgOiBJRGVwZW5kZW5jeVN3aXRjaGVyU2VydmljZQogICAgewogICAgICAgIHByaXZhdGUgc3RhdGljIFJlZ2V4IF9wcm9qZWN0UmVmZXJlbmNlID0gbmV3IFJlZ2V4KEAiPFByb2plY3RSZWZlcmVuY2UgSW5jbHVkZT0iIig/PHByb2plY3RGb2xkZXJQYXRoPi4rKSg/PHByb2plY3RGaWxlTmFtZT5bXjxeXF4vXSspXC4oPzxwcm9qZWN0RmlsZUV4dGVuc2lvbj4uKykiIiIpOwogICAgICAgIHByaXZhdGUgc3RhdGljIFJlZ2V4IF9wYWNrYWdlVmVyc2lvbiA9IG5ldyBSZWdleChAIjxQYWNrYWdlVmVyc2lvbj4oPzxwYWNrYWdlVmVyc2lvbj5bXjxdKyk8L1BhY2thZ2VWZXJzaW9uPiIpOwoKICAgICAgICBwdWJsaWMgdm9pZCBDb252ZXJ0VG9Qcm9qZWN0UmVmZXJlbmNlcyhBYnNvbHV0ZVBhdGggc29sdXRpb25Gb2xkZXIsIEZ1bmM8QWJzb2x1dGVQYXRoLCBzdHJpbmcsIGJvb2w+IHJlZmVyZW5jZVByZWRpY2F0ZSwgYm9vbCBiYWNrdXApCiAgICAgICAgewogICAgICAgICAgICB2YXIgZmlsZXMgPSBzb2x1dGlvbkZvbGRlci5HZXREZXNjZW5kYW50cygiKi5jc3Byb2oiKS5Ub0ltbXV0YWJsZUxpc3QoKTsKCiAgICAgICAgICAgIHZhciBmaWxlc1RvUGFja2FnZXMgPSBuZXcgRGljdGlvbmFyeTxBYnNvbHV0ZVBhdGgsIEZ1bmM8c3RyaW5nLCBzdHJpbmc+PigpOwogICAgICAgICAgICAKICAgICAgICAgICAgZm9yZWFjaCAodmFyIGZpbGUgaW4gZmlsZXMpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHZhciB0ZXh0ID0gZmlsZS5SZWFkQWxsVGV4dCgpOwogICAgICAgICAgICAgICAgdmFyIHBhY2thZ2VWZXJzaW9uID0gX3BhY2thZ2VWZXJzaW9uLk1hdGNoKHRleHQpOwoKICAgICAgICAgICAgICAgIGlmIChwYWNrYWdlVmVyc2lvbi5TdWNjZXNzKQogICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgIHZhciByZWdleCA9IG5ldyBSZWdleCgkIjxQcm9qZWN0UmVmZXJlbmNlIEluY2x1ZGU9XCIuK3tmaWxlLk5hbWV9XCIgKi8+Iik7CgogICAgICAgICAgICAgICAgICAgIHZhciBwYWNrYWdlVmVyc2lvbk51bWJlciA9IHBhY2thZ2VWZXJzaW9uLkdyb3Vwc1sicGFja2FnZVZlcnNpb24iXS5WYWx1ZTsKICAgICAgICAgICAgICAgICAgICBmaWxlc1RvUGFja2FnZXNbZmlsZV0gPSBzdHIgPT4gcmVnZXguUmVwbGFjZShzdHIsICQiPFBhY2thZ2VSZWZlcmVuY2UgSW5jbHVkZT1cIntmaWxlLldpdGhvdXRFeHRlbnNpb24oKX1cIiBWZXJzaW9uPVwie3BhY2thZ2VWZXJzaW9uTnVtYmVyfVwiIC8+Iik7CiAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgIH0KCiAgICAgICAgICAgIGZvcmVhY2ggKHZhciBmaWxlIGluIGZpbGVzKQogICAgICAgICAgICB7CiAgICAgICAgICAgICAgICB2YXIgdGV4dCA9IGZpbGUuUmVhZEFsbFRleHQoKTsKCiAgICAgICAgICAgICAgICBpZiAoIXJlZmVyZW5jZVByZWRpY2F0ZShmaWxlLCB0ZXh0KSkKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAvLyBUT0RPIC0gbWFrZSB0aGlzIGJlaGF2aW9yIGNvbmZpZ3VyYWJsZS4KICAgICAgICAgICAgICAgICAgICBjb250aW51ZTsKICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIAogICAgICAgICAgICAgICAgdmFyIG51bUNoYW5nZXMgPSAwOwogICAgICAgICAgICAgICAgZm9yZWFjaCAodmFyIHBhY2thZ2UgaW4gZmlsZXNUb1BhY2thZ2VzKQogICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgIHZhciBwcmV2VGV4dCA9IHRleHQ7CiAgICAgICAgICAgICAgICAgICAgdGV4dCA9IHBhY2thZ2UuVmFsdWUodGV4dCk7CiAgICAgICAgICAgICAgICAgICAgaWYgKCF0ZXh0LkVxdWFscyhwcmV2VGV4dCkpCiAgICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICAgICBudW1DaGFuZ2VzKys7CiAgICAgICAgICAgICAgICAgICAgICAgIENvbnNvbGUuV3JpdGVMaW5lKCQiQ2hhbmdpbmcge2ZpbGUuV2l0aG91dEV4dGVuc2lvbigpfSdzIGRlcGVuZGVuY3kgb24ge3BhY2thZ2UuS2V5fSBmcm9tIGEgcHJvamVjdCByZWZlcmVuY2UgdG8gYSBwYWNrYWdlIHJlZmVyZW5jZSIpOwogICAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIAogICAgICAgICAgICAgICAgZmlsZS5Xcml0ZUFsbFRleHQodGV4dCk7CiAgICAgICAgICAgIH0KICAgICAgICB9CiAgICAgICAgCiAgICAgICAgcHVibGljIHZvaWQgQ29udmVydFRvUGFja2FnZVJlZmVyZW5jZXMoQWJzb2x1dGVQYXRoIHNvbHV0aW9uRm9sZGVyLCBGdW5jPEFic29sdXRlUGF0aCwgc3RyaW5nLCBib29sPiByZWZlcmVuY2VQcmVkaWNhdGUsIGJvb2wgYmFja3VwKQogICAgICAgIHsKICAgICAgICAgICAgdmFyIGZpbGVzID0gc29sdXRpb25Gb2xkZXIuR2V0RGVzY2VuZGFudHMoIiouY3Nwcm9qIikuVG9JbW11dGFibGVMaXN0KCk7CgogICAgICAgICAgICB2YXIgZmlsZXNUb1BhY2thZ2VzID0gbmV3IERpY3Rpb25hcnk8QWJzb2x1dGVQYXRoLCBGdW5jPHN0cmluZywgQWJzb2x1dGVQYXRoLCBzdHJpbmc+PigpOwogICAgICAgICAgICAKICAgICAgICAgICAgZm9yZWFjaCAodmFyIGZpbGUgaW4gZmlsZXMpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHZhciB0ZXh0ID0gZmlsZS5SZWFkQWxsVGV4dCgpOwogICAgICAgICAgICAgICAgdmFyIHBhY2thZ2VWZXJzaW9uID0gX3BhY2thZ2VWZXJzaW9uLk1hdGNoKHRleHQpOwoKICAgICAgICAgICAgICAgIGlmIChwYWNrYWdlVmVyc2lvbi5TdWNjZXNzKQogICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgIHZhciByZWdleCA9IG5ldyBSZWdleCgkIjxQYWNrYWdlUmVmZXJlbmNlIEluY2x1ZGU9XCJ7UGF0aC5HZXRGaWxlTmFtZVdpdGhvdXRFeHRlbnNpb24oZmlsZS5Ub1N0cmluZygpKX1cIlteL10rLz4iKTsKCiAgICAgICAgICAgICAgICAgICAgZmlsZXNUb1BhY2thZ2VzW2ZpbGVdID0gKGNzcHJvakZpbGVUZXh0LCByZWxhdGl2ZVRvKSA9PgogICAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAgICAgdmFyIHBhdGggPSBmaWxlLlJlbGF0aXZlVG8ocmVsYXRpdmVUbyk7CgogICAgICAgICAgICAgICAgICAgICAgICB2YXIgcmVzdWx0ID0gcmVnZXguUmVwbGFjZShjc3Byb2pGaWxlVGV4dCwKICAgICAgICAgICAgICAgICAgICAgICAgICAgICQiPFBhY2thZ2VSZWZlcmVuY2UgSW5jbHVkZT1cIntQYXRoLkdldEZpbGVOYW1lV2l0aG91dEV4dGVuc2lvbihmaWxlLlRvU3RyaW5nKCkpfVwiIFZlcnNpb249XCJ7cGFja2FnZVZlcnNpb24uR3JvdXBzWyJwYWNrYWdlVmVyc2lvbiJdLlZhbHVlfVwiIC8+Iik7CiAgICAgICAgICAgICAgICAgICAgICAgIHJldHVybiByZXN1bHQ7CiAgICAgICAgICAgICAgICAgICAgfTsKICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgfQoKICAgICAgICAgICAgZm9yZWFjaCAodmFyIGZpbGUgaW4gZmlsZXMpCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgIHZhciB0ZXh0ID0gZmlsZS5SZWFkQWxsVGV4dCgpOwoKICAgICAgICAgICAgICAgIHZhciBudW1DaGFuZ2VzID0gMDsKICAgICAgICAgICAgICAgIGZvcmVhY2ggKHZhciBwYWNrYWdlIGluIGZpbGVzVG9QYWNrYWdlcykKICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICB2YXIgcHJldlRleHQgPSB0ZXh0OwogICAgICAgICAgICAgICAgICAgIHRleHQgPSBwYWNrYWdlLlZhbHVlKHRleHQsIGZpbGUuUGFyZW50KCkpOwogICAgICAgICAgICAgICAgICAgIGlmICghdGV4dC5FcXVhbHMocHJldlRleHQpKQogICAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAgICAgbnVtQ2hhbmdlcysrOwogICAgICAgICAgICAgICAgICAgICAgICBDb25zb2xlLldyaXRlTGluZSgkIlVwZGF0aW5nIHtQYXRoLkdldEZpbGVOYW1lV2l0aG91dEV4dGVuc2lvbihmaWxlLlRvU3RyaW5nKCkpfSdzIGRlcGVuZGVuY3kgb24ge1BhdGguR2V0RmlsZU5hbWVXaXRob3V0RXh0ZW5zaW9uKHBhY2thZ2UuS2V5LlRvU3RyaW5nKCkpfSIpOwogICAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIAogICAgICAgICAgICAgICAgZmlsZS5Xcml0ZUFsbFRleHQodGV4dCk7CiAgICAgICAgICAgIH0KICAgICAgICB9CiAgICB9Cn0="
    }
  },
  "curation_script": "from pathlib import Path\nimport subprocess,json,hashlib,re\nROOT=Path.cwd();BASE=ROOT/'target/local-agent/history';REPOS=BASE/'repos'\ndef git(repo,*args):return subprocess.check_output(['git','-C',str(REPOS/repo),*args],text=True,encoding='utf-8-sig')\ndef method(source,name):\n # Curated signatures in inspected historical source, not a general C# parser.\n match=re.search(r'^        (?:public|private) (?:static )?[^\\n]*\\b'+re.escape(name)+r'(?:<[^\\n]+>)?\\(',source,re.M)\n if not match:raise ValueError(name)\n start=source.index('{',match.start());depth=1;i=start+1\n # Inspected methods have balanced braces in interpolation; scan retains them.\n while depth:\n  depth += (source[i]=='{')-(source[i]=='}');i+=1\n return source[match.start():i],source[:match.start()].count('\\n')+1\nUSINGS='using System;\\nusing System.Collections.Generic;\\nusing System.Collections.Immutable;\\nusing System.Linq;\\nusing System.IO;\\nusing System.Text.RegularExpressions;\\nusing System.Runtime.CompilerServices;\\n'\nASSERT='''\n    static void Equal<T>(T actual,T expected) { if(!object.Equals(actual,expected)) throw new Exception($\"Expected {expected}, got {actual}\"); }\n    static void True(bool value) { if(!value) throw new Exception(\"Assertion failed\"); }\n    static void Seq<T>(IEnumerable<T> actual,params T[] expected) { True(actual.SequenceEqual(expected)); }\n'''\ncases=[]\ndef add(id,repo,rev,path,name,prompt,public,hidden,support='',before_rev=None,before_name=None,kind='completion'):\n sha=git(repo,'rev-parse',rev).strip();parent=git(repo,'rev-parse',sha+'^').strip() if git(repo,'rev-list','--parents','-n','1',sha).count(' ') else None\n source=git(repo,'show',sha+':'+path);answer,line=method(source,name)\n if before_rev:\n  before,_=method(git(repo,'show',before_rev+':'+path),before_name or name)\n else:\n  before=answer[:answer.index('{')+1]+'\\n            throw new NotImplementedException();\\n        }'\n metadata=json.loads(git(repo,'show','-s','--format={\"author_name\":\"%an\",\"author_email\":\"%ae\",\"author_date\":\"%aI\",\"committer_date\":\"%cI\"}',sha))\n cases.append(dict(id=id,kind=kind,repository='nloum/'+repo,commit=sha,parent=parent,path=path,method=name,source_line=line,normalized_source_sha256=hashlib.sha256(source.encode()).hexdigest(),source_blob_oid=git(repo,'rev-parse',sha+':'+path).strip(),source_raw_sha256=hashlib.sha256(subprocess.check_output(['git','-C',str(REPOS/repo),'show',sha+':'+path])).hexdigest(),answer_sha256=hashlib.sha256(answer.encode()).hexdigest(),author=metadata,prompt=prompt,prompt_origin='Retrospective task contract curated from the commit and inspected behavior; not an original historical request',before=before,answer=answer,usings=USINGS,support=support,assertions=ASSERT,public_tests=public,hidden_tests=hidden,static_owner=(repo=='CodeIO'),projection='Historical method with normalized newlines inside a reduced C# fixture; supplied collaborators are explicit test doubles. This is not a full original-project build.'))\nrepo='CodeIO';rev='c850a1b';path='src/FluentSourceGenerators/Utilities.cs';src=git(repo,'show',rev+':'+path)\nhelper=lambda name,source=src:method(source,name)[0]\nadd('codeio-exception-factory',repo,rev,path,'MakeException','Refactor ThrowException into public static InvalidOperationException MakeException. It must return, rather than throw, the exception. Preserve optional CallerFilePath, CallerLineNumber and CallerMemberName attributes and append exactly \" (file:line in member)\" to the supplied message. The caller chooses whether to throw.',\n'var e=MakeException(\"bad\", \"sample.cs\", 17, \"Build\"); Equal(e.Message,\"bad (sample.cs:17 in Build)\");',\n'var e=MakeException(\"\", \"x\", 0, \"\"); Equal(e.Message,\" (x:0 in )\"); var automatic=MakeException(\"automatic\"); True(automatic.Message.Contains(\"Program.cs:\")); True(automatic.Message.EndsWith(\" in Hidden)\"));',before_rev=rev+'^',before_name='ThrowException',kind='refactor')\nrev='a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631';path='FluentSourceGenerators/Utilities.cs';src=git(repo,'show',rev+':'+path)\nadd('codeio-strip-type-arguments',repo,rev,path,'GetWithoutTypeArguments','Implement GetWithoutTypeArguments: return the prefix before the first < character, or the original string if none. Keep whitespace unchanged. This helper does not parse or validate C# types.',\n'Equal(GetWithoutTypeArguments(\"Map<string, int>\"),\"Map\"); Equal(GetWithoutTypeArguments(\"Int32\"),\"Int32\");',\n'Equal(GetWithoutTypeArguments(\"\"),\"\"); Equal(GetWithoutTypeArguments(\"<T>\"),\"\"); Equal(GetWithoutTypeArguments(\" A <B<C>>\"),\" A \"); Equal(GetWithoutTypeArguments(\"A>B\"),\"A>B\");')\nadd('codeio-variable-name',repo,rev,path,'GenerateVariableName','Implement GenerateVariableName. If isInterface is true remove the first character. Remove the first < and everything after it when present. Apply the supplied Camelize helper to the remaining text. Assume interface names are nonempty; preserve the behavior of the supplied helpers.',\n'Equal(GenerateVariableName(\"IReadOnlyList<T>\",true),\"readOnlyList\"); Equal(GenerateVariableName(\"DataStore\",false),\"dataStore\");',\n'Equal(GenerateVariableName(\"IFoo_Bar<T,U>\",true),\"fooBar\"); Equal(GenerateVariableName(\"\",false),\"\"); Equal(GenerateVariableName(\"I\",true),\"\"); Equal(GenerateVariableName(\"some name\",false),\"someName\");',support=helper('Pascalize')+'\\n'+helper('Camelize'))\nadd('codeio-cartesian-product',repo,rev,path,'CalcCombinationsOfOneFromEach','Implement a lazy Cartesian product over restartable finite sequences. Each result is an ImmutableList<TItem>. The first input dimension advances fastest. Empty outer input yields one empty combination. Any empty inner sequence throws ArgumentException when the result is enumerated. Do not change the signature.',\n'var got=CalcCombinationsOfOneFromEach(new[]{new[]{1,2},new[]{3,4}}).Select(x=>string.Join(\",\",x)); Seq(got,\"1,3\",\"2,3\",\"1,4\",\"2,4\");',\n'var empty=CalcCombinationsOfOneFromEach(Array.Empty<IEnumerable<int>>()).ToList(); Equal(empty.Count,1); Equal(empty[0].Count,0); bool threw=false; try { CalcCombinationsOfOneFromEach(new[]{Array.Empty<int>()}).ToList(); } catch(ArgumentException) { threw=true; } True(threw); Seq(CalcCombinationsOfOneFromEach(new[]{new[]{\"a\",\"b\"},new[]{\"x\"},new[]{\"y\",\"z\"}}).Select(x=>string.Join(\"\",x)),\"axy\",\"bxy\",\"axz\",\"bxz\");')\nletter=r'''\n    public class Box<T> { public T Value; }\n    class ManualClassification { public int X; public string Letter; }\n    private string _folderPath=\"images\";\n    private List<ManualClassification> manualClassifications=new();\n    private List<int> xValues=new();\n    public Box<string> ImagePath=new();\n    public Box<string> Letter=new();\n    private static Regex _imageFileNameRegex=new Regex(@\"image(\\d+)\\.png\");\n'''\nlrepo='letter-classifier';lrev='c97dd60';lpath='MainViewModel.cs';lsrc=git(lrepo,'show',lrev+':'+lpath)\nadd('letters-next-image',lrepo,lrev,lpath,'PrepareToClassifyNextImage','Implement PrepareToClassifyNextImage. Visit xValues in their existing order; choose the first X without a manual classification, set ImagePath.Value to _folderPath + \"\\\\\\\\image\" + X + \".png\", clear Letter.Value, and stop. If all values are classified (or the list is empty), leave both properties unchanged. Use the supplied small collaborators.',\nr'var s=new Subject(); s.xValues.AddRange(new[]{9,2}); s.PrepareToClassifyNextImage(); Equal(s.ImagePath.Value,@\"images\\image9.png\"); Equal(s.Letter.Value,\"\");',\nr'var s=new Subject(); s.xValues.AddRange(new[]{9,2,7}); s.manualClassifications.Add(new(){X=9}); s.PrepareToClassifyNextImage(); Equal(s.ImagePath.Value,@\"images\\image2.png\"); s.manualClassifications.AddRange(new[]{new ManualClassification(){X=2},new ManualClassification(){X=7}}); s.Letter.Value=\"keep\"; s.PrepareToClassifyNextImage(); Equal(s.Letter.Value,\"keep\"); var e=new Subject(); e.ImagePath.Value=\"old\"; e.Letter.Value=\"label\"; e.PrepareToClassifyNextImage(); Equal(e.ImagePath.Value,\"old\"); Equal(e.Letter.Value,\"label\");',support=letter)\nadd('letters-record-label',lrepo,lrev,lpath,'LabelImage','Implement LabelImage. Parse the integer captured by _imageFileNameRegex from the basename of ImagePath.Value. Append a ManualClassification with that X and the current Letter.Value, then call the supplied PrepareToClassifyNextImage. Assume a valid matching image path. Keep the label string exactly as entered.',\nr'var s=new Subject(); s.xValues.AddRange(new[]{7,4}); s.ImagePath.Value=@\"images\\image7.png\"; s.Letter.Value=\"Q\"; s.LabelImage(); Equal(s.manualClassifications.Count,1); Equal(s.manualClassifications[0].X,7); Equal(s.manualClassifications[0].Letter,\"Q\"); Equal(s.ImagePath.Value,@\"images\\image4.png\");',\nr'var s=new Subject(); s.xValues.Add(123); s.ImagePath.Value=\"/images/image123.png\"; s.Letter.Value=\"a b\"; s.LabelImage(); Equal(s.manualClassifications.Single().X,123); Equal(s.manualClassifications.Single().Letter,\"a b\"); Equal(s.Letter.Value,\"a b\");',support=letter+'\\n'+method(lsrc,'PrepareToClassifyNextImage')[0])\npaths='''\n    public sealed class AbsolutePath {\n        public static readonly Dictionary<string,string> Files=new();\n        public static readonly List<string> Writes=new();\n        readonly string value;\n        public AbsolutePath(string value) { this.value=value; }\n        public string Name => Path.GetFileName(value);\n        public IEnumerable<AbsolutePath> GetDescendants(string pattern) => Files.Keys.Where(x=>x.StartsWith(value+\"/\") && x.EndsWith(\".csproj\")).Select(x=>new AbsolutePath(x)).ToList();\n        public string ReadAllText()=>Files[value];\n        public void WriteAllText(string text) { Files[value]=text; Writes.Add(value); }\n        public AbsolutePath Parent()=>new AbsolutePath(Path.GetDirectoryName(value));\n        public AbsolutePath WithoutExtension()=>new AbsolutePath(Path.ChangeExtension(value,null));\n        public bool HasExtension(string extension)=>value.EndsWith(extension);\n        public string RelativeTo(AbsolutePath parent)=>Path.GetRelativePath(parent.value,value);\n        public override string ToString()=>value;\n        public override bool Equals(object other)=>other is AbsolutePath p && p.value==value;\n        public override int GetHashCode()=>value.GetHashCode();\n    }\n    private static Regex _packageVersion = new Regex(@\"<PackageVersion>(?<packageVersion>[^<]+)</PackageVersion>\");\n    static void Setup(params (string,string)[] files) { AbsolutePath.Files.Clear(); AbsolutePath.Writes.Clear(); foreach(var (name,text) in files) AbsolutePath.Files[name]=text; }\n'''\nrepo='PackageProjectDependencySwitcher';rev='9fa4a1a';path='src/PackageProjectDependencySwitcher/DependencySwitcherService.cs'\nadd('dependencies-to-project',repo,rev,path,'ConvertToProjectReferences','Repair ConvertToProjectReferences so it converts PackageReference entries matching local versioned projects into ProjectReference entries relative to each consumer directory. Do not modify a project to reference itself; preserve unrelated entries and do not write unchanged files. Keep the public signature; referencePredicate and backup are unused in this historical contract. Inputs use simple self-closing XML entries and local projects have PackageVersion tags. Use supplied AbsolutePath collaborator.',\n'''Setup((\"/s/Lib/Lib.csproj\",\"<Project><PackageVersion>1.2</PackageVersion></Project>\"),(\"/s/App/App.csproj\",\"<Project><PackageReference Include=\\\\\"Lib\\\\\" Version=\\\\\"0.1\\\\\" /></Project>\")); new Subject().ConvertToProjectReferences(new AbsolutePath(\"/s\"),(p,t)=>true,false); True(AbsolutePath.Files[\"/s/App/App.csproj\"].Contains(\"<ProjectReference Include=\\\\\"../Lib/Lib.csproj\\\\\" />\")); Equal(AbsolutePath.Writes.Count,1);''',\n'''Setup((\"/s/A/A.csproj\",\"<Project><PackageVersion>3</PackageVersion><PackageReference Include=\\\\\"A\\\\\" Version=\\\\\"1\\\\\" /></Project>\"),(\"/s/B/B.csproj\",\"<Project><PackageReference Include=\\\\\"External\\\\\" Version=\\\\\"9\\\\\" /></Project>\")); new Subject().ConvertToProjectReferences(new AbsolutePath(\"/s\"),(p,t)=>true,false); Equal(AbsolutePath.Writes.Count,0); True(AbsolutePath.Files[\"/s/B/B.csproj\"].Contains(\"External\"));''',support=paths,before_rev=rev+'^',kind='repair')\nadd('dependencies-to-package',repo,rev,path,'ConvertToPackageReferences','Repair ConvertToPackageReferences: replace ProjectReference entries targeting local versioned projects with PackageReference entries using the target filename without extension and its PackageVersion. Skip consumers whose project name without .csproj ends in .Test. Preserve unrelated content. Keep the signature and supplied AbsolutePath API. referencePredicate and backup are unused in this historical contract. Inputs use simple self-closing XML entries.',\n'''Setup((\"/s/Lib/Lib.csproj\",\"<Project><PackageVersion>1.2</PackageVersion></Project>\"),(\"/s/App/App.csproj\",\"<Project><ProjectReference Include=\\\\\"../Lib/Lib.csproj\\\\\" /></Project>\")); new Subject().ConvertToPackageReferences(new AbsolutePath(\"/s\"),(p,t)=>true,false); True(AbsolutePath.Files[\"/s/App/App.csproj\"].Contains(\"<PackageReference Include=\\\\\"Lib\\\\\" Version=\\\\\"1.2\\\\\" />\"));''',\n'''Setup((\"/s/Lib/Lib.csproj\",\"<Project><PackageVersion>7</PackageVersion></Project>\"),(\"/s/App.Test/App.Test.csproj\",\"<Project><ProjectReference Include=\\\\\"../Lib/Lib.csproj\\\\\" /></Project>\"),(\"/s/App/App.csproj\",\"<Project><ProjectReference Include=\\\\\"../Other/Other.csproj\\\\\" /></Project>\")); new Subject().ConvertToPackageReferences(new AbsolutePath(\"/s\"),(p,t)=>true,false); True(!AbsolutePath.Writes.Contains(\"/s/App.Test/App.Test.csproj\")); True(AbsolutePath.Files[\"/s/App/App.csproj\"].Contains(\"Other/Other.csproj\"));''',support=paths,before_rev=rev+'^',kind='repair')\nfor c in cases:\n c['commit_url']='https://github.com/'+c['repository']+'/commit/'+c['commit']\n c['source_url']='https://github.com/'+c['repository']+'/blob/'+c['commit']+'/'+c['path']+'#L'+str(c['source_line'])\n(ROOT/'target/local-agent/history/dataset.json').write_text(json.dumps(dict(version=1,cutoff_exclusive='2021-01-01',cases=cases),indent=2)+'\\n')\nprint([(c['id'],len(c['answer']),len(c['support'])) for c in cases])\n",
  "fixture_failures": [
    {
      "case": "codeio-exception-factory",
      "reference_public": {
        "passed": true,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:03.04\n",
          "output_truncated": false,
          "elapsed_seconds": 4.072628775000339,
          "timeout": false
        },
        "run": {
          "exit_code": 0,
          "output": "CHECKS_PASSED\n",
          "output_truncated": false,
          "elapsed_seconds": 0.26435095199849457,
          "timeout": false
        }
      },
      "reference_hidden": {
        "passed": true,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.59\n",
          "output_truncated": false,
          "elapsed_seconds": 3.019184614000551,
          "timeout": false
        },
        "run": {
          "exit_code": 0,
          "output": "CHECKS_PASSED\n",
          "output_truncated": false,
          "elapsed_seconds": 0.26438275200052885,
          "timeout": false
        }
      },
      "before_public": {
        "passed": false,
        "stage": "compile",
        "build": {
          "exit_code": 1,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(22,30): error CS0103: The name 'MakeException' does not exist in the current context [/work/Study.csproj]\n/work/Program.cs(23,30): error CS0103: The name 'MakeException' does not exist in the current context [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(22,30): error CS0103: The name 'MakeException' does not exist in the current context [/work/Study.csproj]\n/work/Program.cs(23,30): error CS0103: The name 'MakeException' does not exist in the current context [/work/Study.csproj]\n    0 Warning(s)\n    2 Error(s)\n\nTime Elapsed 00:00:02.49\n",
          "output_truncated": false,
          "elapsed_seconds": 2.9186777000068105,
          "timeout": false
        },
        "run": null
      },
      "eligible": true
    },
    {
      "case": "codeio-strip-type-arguments",
      "reference_public": {
        "passed": true,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.63\n",
          "output_truncated": false,
          "elapsed_seconds": 3.0690264149961877,
          "timeout": false
        },
        "run": {
          "exit_code": 0,
          "output": "CHECKS_PASSED\n",
          "output_truncated": false,
          "elapsed_seconds": 0.2644121979974443,
          "timeout": false
        }
      },
      "reference_hidden": {
        "passed": true,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.62\n",
          "output_truncated": false,
          "elapsed_seconds": 3.0689859080011956,
          "timeout": false
        },
        "run": {
          "exit_code": 0,
          "output": "CHECKS_PASSED\n",
          "output_truncated": false,
          "elapsed_seconds": 0.3147184539993759,
          "timeout": false
        }
      },
      "before_public": {
        "passed": false,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.62\n",
          "output_truncated": false,
          "elapsed_seconds": 3.068892452,
          "timeout": false
        },
        "run": {
          "exit_code": 1,
          "output": "NotImplementedException: The method or operation is not implemented.\n",
          "output_truncated": false,
          "elapsed_seconds": 0.3143941030066344,
          "timeout": false
        }
      },
      "eligible": true
    },
    {
      "case": "codeio-variable-name",
      "reference_public": {
        "passed": false,
        "stage": "compile",
        "build": {
          "exit_code": 1,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(9,14): error CS1106: Extension method must be defined in a non-generic static class [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(9,14): error CS1106: Extension method must be defined in a non-generic static class [/work/Study.csproj]\n    0 Warning(s)\n    1 Error(s)\n\nTime Elapsed 00:00:02.27\n",
          "output_truncated": false,
          "elapsed_seconds": 2.7193456920067547,
          "timeout": false
        },
        "run": null
      },
      "reference_hidden": {
        "passed": false,
        "stage": "compile",
        "build": {
          "exit_code": 1,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(9,14): error CS1106: Extension method must be defined in a non-generic static class [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(9,14): error CS1106: Extension method must be defined in a non-generic static class [/work/Study.csproj]\n    0 Warning(s)\n    1 Error(s)\n\nTime Elapsed 00:00:02.24\n",
          "output_truncated": false,
          "elapsed_seconds": 2.6682966059961473,
          "timeout": false
        },
        "run": null
      },
      "before_public": {
        "passed": false,
        "stage": "compile",
        "build": {
          "exit_code": 1,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(9,14): error CS1106: Extension method must be defined in a non-generic static class [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(9,14): error CS1106: Extension method must be defined in a non-generic static class [/work/Study.csproj]\n    0 Warning(s)\n    1 Error(s)\n\nTime Elapsed 00:00:02.22\n",
          "output_truncated": false,
          "elapsed_seconds": 2.668232285002887,
          "timeout": false
        },
        "run": null
      },
      "eligible": false
    },
    {
      "case": "codeio-cartesian-product",
      "reference_public": {
        "passed": true,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.73\n",
          "output_truncated": false,
          "elapsed_seconds": 3.1690814760004287,
          "timeout": false
        },
        "run": {
          "exit_code": 0,
          "output": "CHECKS_PASSED\n",
          "output_truncated": false,
          "elapsed_seconds": 0.2642516159976367,
          "timeout": false
        }
      },
      "reference_hidden": {
        "passed": true,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.75\n",
          "output_truncated": false,
          "elapsed_seconds": 3.169233852997422,
          "timeout": false
        },
        "run": {
          "exit_code": 0,
          "output": "CHECKS_PASSED\n",
          "output_truncated": false,
          "elapsed_seconds": 0.26439471499907086,
          "timeout": false
        }
      },
      "before_public": {
        "passed": false,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.63\n",
          "output_truncated": false,
          "elapsed_seconds": 3.069980141000997,
          "timeout": false
        },
        "run": {
          "exit_code": 1,
          "output": "NotImplementedException: The method or operation is not implemented.\n",
          "output_truncated": false,
          "elapsed_seconds": 0.31441820799955167,
          "timeout": false
        }
      },
      "eligible": true
    },
    {
      "case": "letters-next-image",
      "reference_public": {
        "passed": false,
        "stage": "compile",
        "build": {
          "exit_code": 1,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n    0 Warning(s)\n    2 Error(s)\n\nTime Elapsed 00:00:02.26\n",
          "output_truncated": false,
          "elapsed_seconds": 2.6686615939979674,
          "timeout": false
        },
        "run": null
      },
      "reference_hidden": {
        "passed": false,
        "stage": "compile",
        "build": {
          "exit_code": 1,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n    0 Warning(s)\n    2 Error(s)\n\nTime Elapsed 00:00:02.27\n",
          "output_truncated": false,
          "elapsed_seconds": 2.718276687999605,
          "timeout": false
        },
        "run": null
      },
      "before_public": {
        "passed": false,
        "stage": "compile",
        "build": {
          "exit_code": 1,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n    0 Warning(s)\n    2 Error(s)\n\nTime Elapsed 00:00:02.27\n",
          "output_truncated": false,
          "elapsed_seconds": 2.7191612949973205,
          "timeout": false
        },
        "run": null
      },
      "eligible": false
    },
    {
      "case": "letters-record-label",
      "reference_public": {
        "passed": false,
        "stage": "compile",
        "build": {
          "exit_code": 1,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n    0 Warning(s)\n    2 Error(s)\n\nTime Elapsed 00:00:02.34\n",
          "output_truncated": false,
          "elapsed_seconds": 2.7685047170016333,
          "timeout": false
        },
        "run": null
      },
      "reference_hidden": {
        "passed": false,
        "stage": "compile",
        "build": {
          "exit_code": 1,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n    0 Warning(s)\n    2 Error(s)\n\nTime Elapsed 00:00:02.29\n",
          "output_truncated": false,
          "elapsed_seconds": 2.71856757599744,
          "timeout": false
        },
        "run": null
      },
      "before_public": {
        "passed": false,
        "stage": "compile",
        "build": {
          "exit_code": 1,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(16,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.ImagePath' [/work/Study.csproj]\n/work/Program.cs(17,24): error CS0052: Inconsistent accessibility: field type 'Subject.Box<string>' is less accessible than field 'Subject.Letter' [/work/Study.csproj]\n    0 Warning(s)\n    2 Error(s)\n\nTime Elapsed 00:00:02.23\n",
          "output_truncated": false,
          "elapsed_seconds": 2.668233667005552,
          "timeout": false
        },
        "run": null
      },
      "eligible": false
    },
    {
      "case": "dependencies-to-project",
      "reference_public": {
        "passed": true,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.64\n",
          "output_truncated": false,
          "elapsed_seconds": 3.069014081003843,
          "timeout": false
        },
        "run": {
          "exit_code": 0,
          "output": "Changing App's dependency on Lib from a package reference to a project reference\nCHECKS_PASSED\n",
          "output_truncated": false,
          "elapsed_seconds": 0.3145434239995666,
          "timeout": false
        }
      },
      "reference_hidden": {
        "passed": true,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.68\n",
          "output_truncated": false,
          "elapsed_seconds": 3.1201977000018815,
          "timeout": false
        },
        "run": {
          "exit_code": 0,
          "output": "CHECKS_PASSED\n",
          "output_truncated": false,
          "elapsed_seconds": 0.3144305120003992,
          "timeout": false
        }
      },
      "before_public": {
        "passed": false,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.68\n",
          "output_truncated": false,
          "elapsed_seconds": 3.1194604109987267,
          "timeout": false
        },
        "run": {
          "exit_code": 1,
          "output": "Exception: Assertion failed\n",
          "output_truncated": false,
          "elapsed_seconds": 0.31441515299957246,
          "timeout": false
        }
      },
      "eligible": true
    },
    {
      "case": "dependencies-to-package",
      "reference_public": {
        "passed": true,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.60\n",
          "output_truncated": false,
          "elapsed_seconds": 3.0691006740016746,
          "timeout": false
        },
        "run": {
          "exit_code": 0,
          "output": "Changing App's dependency on Lib from a project reference to a package reference\nCHECKS_PASSED\n",
          "output_truncated": false,
          "elapsed_seconds": 0.31440524400386494,
          "timeout": false
        }
      },
      "reference_hidden": {
        "passed": true,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.70\n",
          "output_truncated": false,
          "elapsed_seconds": 3.169005403004121,
          "timeout": false
        },
        "run": {
          "exit_code": 0,
          "output": "CHECKS_PASSED\n",
          "output_truncated": false,
          "elapsed_seconds": 0.3147134650062071,
          "timeout": false
        }
      },
      "before_public": {
        "passed": false,
        "stage": "execute",
        "build": {
          "exit_code": 0,
          "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.72\n",
          "output_truncated": false,
          "elapsed_seconds": 3.1690761869976996,
          "timeout": false
        },
        "run": {
          "exit_code": 1,
          "output": "Exception: Assertion failed\n",
          "output_truncated": false,
          "elapsed_seconds": 0.2643656610016478,
          "timeout": false
        }
      },
      "eligible": true
    }
  ],
  "qualification_note": "The first fixture wrapper failed because of static extension-method ownership and Box visibility. Corrected wrappers were rerun; the same historical method bodies were retained. Later source provenance moved completion cases to their first introduction after verifying all executable fixture inputs remained identical.",
  "exclusions": {
    "CSharpCream.NET": "Initial code drops only; no small historical repair selected and original third-party authorship requires review.",
    "JoyOfCooking2018": "Examined changes were recipe data or filenames, not executable code tasks for this C# pilot.",
    "Replace-Package": "Empty repository in discovery; no eligible implementation.",
    "CodeIO": "Broad generator changes requiring Roslyn and original dependent projects were not treated as verified full-repository tasks.",
    "letter-classifier": "WPF/CSV installation and UI behavior excluded; only two method projections with explicit collaborators are verified.",
    "PackageProjectDependencySwitcher": "Packaging/version/CI changes excluded; two target methods share one repair commit and are dependent observations."
  }
}
</build:entry>

<build:entry path="docs/experiments/historical-code/2026-11-01/references.json">
[
  {
    "case": "codeio-exception-factory",
    "reference_public": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.62\n",
        "output_truncated": false,
        "elapsed_seconds": 3.1203616890052217,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.31456501499633305,
        "timeout": false
      }
    },
    "reference_hidden": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.66\n",
        "output_truncated": false,
        "elapsed_seconds": 3.119281563995173,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.2644215959953726,
        "timeout": false
      }
    },
    "before_public": {
      "passed": false,
      "stage": "compile",
      "build": {
        "exit_code": 1,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(22,30): error CS0103: The name 'MakeException' does not exist in the current context [/work/Study.csproj]\n/work/Program.cs(23,30): error CS0103: The name 'MakeException' does not exist in the current context [/work/Study.csproj]\n\nBuild FAILED.\n\n/work/Program.cs(22,30): error CS0103: The name 'MakeException' does not exist in the current context [/work/Study.csproj]\n/work/Program.cs(23,30): error CS0103: The name 'MakeException' does not exist in the current context [/work/Study.csproj]\n    0 Warning(s)\n    2 Error(s)\n\nTime Elapsed 00:00:02.48\n",
        "output_truncated": false,
        "elapsed_seconds": 2.918786735004687,
        "timeout": false
      },
      "run": null
    },
    "eligible": true,
    "fixture_sha256": "700a8c34a9f9108a59c7464d6155cdd8c91a3b587fd7e0f92652bf81ffdb84cc"
  },
  {
    "case": "codeio-strip-type-arguments",
    "reference_public": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.55\n",
        "output_truncated": false,
        "elapsed_seconds": 3.018730367999524,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.31482851200416917,
        "timeout": false
      }
    },
    "reference_hidden": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.61\n",
        "output_truncated": false,
        "elapsed_seconds": 3.0695102359968587,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.3145218739955453,
        "timeout": false
      }
    },
    "before_public": {
      "passed": false,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.60\n",
        "output_truncated": false,
        "elapsed_seconds": 3.0714038939986494,
        "timeout": false
      },
      "run": {
        "exit_code": 1,
        "output": "NotImplementedException: The method or operation is not implemented.\n",
        "output_truncated": false,
        "elapsed_seconds": 0.3646189350038185,
        "timeout": false
      }
    },
    "eligible": true,
    "fixture_sha256": "e7206374dfe79e8dbbc2a3baae206ea44b4f1542145a1e36a113573edaebd232"
  },
  {
    "case": "codeio-variable-name",
    "reference_public": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.69\n",
        "output_truncated": false,
        "elapsed_seconds": 3.119940285003395,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.3143475149990991,
        "timeout": false
      }
    },
    "reference_hidden": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.63\n",
        "output_truncated": false,
        "elapsed_seconds": 3.0718743699981133,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.3144508900004439,
        "timeout": false
      }
    },
    "before_public": {
      "passed": false,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.66\n",
        "output_truncated": false,
        "elapsed_seconds": 3.119061520003015,
        "timeout": false
      },
      "run": {
        "exit_code": 1,
        "output": "NotImplementedException: The method or operation is not implemented.\n",
        "output_truncated": false,
        "elapsed_seconds": 0.31458566400397103,
        "timeout": false
      }
    },
    "eligible": true,
    "fixture_sha256": "b59f5af9170c90384667e43f6998f89de21f7df003641753138081a6ed7bd568"
  },
  {
    "case": "codeio-cartesian-product",
    "reference_public": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.71\n",
        "output_truncated": false,
        "elapsed_seconds": 3.170589717999974,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.31443656400369946,
        "timeout": false
      }
    },
    "reference_hidden": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.75\n",
        "output_truncated": false,
        "elapsed_seconds": 3.1716322120046243,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.31443929800298065,
        "timeout": false
      }
    },
    "before_public": {
      "passed": false,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.63\n",
        "output_truncated": false,
        "elapsed_seconds": 3.070217719003267,
        "timeout": false
      },
      "run": {
        "exit_code": 1,
        "output": "NotImplementedException: The method or operation is not implemented.\n",
        "output_truncated": false,
        "elapsed_seconds": 0.31442617400171,
        "timeout": false
      }
    },
    "eligible": true,
    "fixture_sha256": "7a1706e4543a5727885e03e319fe123cff57def83bf15c2be8ab754522daa3a4"
  },
  {
    "case": "letters-next-image",
    "reference_public": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(12,45): warning CS0649: Field 'Subject.ManualClassification.X' is never assigned to, and will always have its default value 0 [/work/Study.csproj]\n/work/Program.cs(12,62): warning CS0649: Field 'Subject.ManualClassification.Letter' is never assigned to, and will always have its default value null [/work/Study.csproj]\n\nBuild succeeded.\n\n/work/Program.cs(12,45): warning CS0649: Field 'Subject.ManualClassification.X' is never assigned to, and will always have its default value 0 [/work/Study.csproj]\n/work/Program.cs(12,62): warning CS0649: Field 'Subject.ManualClassification.Letter' is never assigned to, and will always have its default value null [/work/Study.csproj]\n    2 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.70\n",
        "output_truncated": false,
        "elapsed_seconds": 3.219500725004764,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.3146053410018794,
        "timeout": false
      }
    },
    "reference_hidden": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(12,62): warning CS0649: Field 'Subject.ManualClassification.Letter' is never assigned to, and will always have its default value null [/work/Study.csproj]\n\nBuild succeeded.\n\n/work/Program.cs(12,62): warning CS0649: Field 'Subject.ManualClassification.Letter' is never assigned to, and will always have its default value null [/work/Study.csproj]\n    1 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.74\n",
        "output_truncated": false,
        "elapsed_seconds": 3.2196314519969746,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.3146180559997447,
        "timeout": false
      }
    },
    "before_public": {
      "passed": false,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(12,62): warning CS0649: Field 'Subject.ManualClassification.Letter' is never assigned to, and will always have its default value null [/work/Study.csproj]\n/work/Program.cs(12,45): warning CS0649: Field 'Subject.ManualClassification.X' is never assigned to, and will always have its default value 0 [/work/Study.csproj]\n/work/Program.cs(13,20): warning CS0414: The field 'Subject._folderPath' is assigned but its value is never used [/work/Study.csproj]\n\nBuild succeeded.\n\n/work/Program.cs(12,62): warning CS0649: Field 'Subject.ManualClassification.Letter' is never assigned to, and will always have its default value null [/work/Study.csproj]\n/work/Program.cs(12,45): warning CS0649: Field 'Subject.ManualClassification.X' is never assigned to, and will always have its default value 0 [/work/Study.csproj]\n/work/Program.cs(13,20): warning CS0414: The field 'Subject._folderPath' is assigned but its value is never used [/work/Study.csproj]\n    3 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.70\n",
        "output_truncated": false,
        "elapsed_seconds": 3.169937178005057,
        "timeout": false
      },
      "run": {
        "exit_code": 1,
        "output": "NotImplementedException: The method or operation is not implemented.\n",
        "output_truncated": false,
        "elapsed_seconds": 0.314484372996958,
        "timeout": false
      }
    },
    "eligible": true,
    "fixture_sha256": "81b766bf84022c00569f781c959785133363fff14b254e1009b18bc64d84a381"
  },
  {
    "case": "letters-record-label",
    "reference_public": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.67\n",
        "output_truncated": false,
        "elapsed_seconds": 3.1220075499950326,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.31439810099982424,
        "timeout": false
      }
    },
    "reference_hidden": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.72\n",
        "output_truncated": false,
        "elapsed_seconds": 3.169745127001079,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.3149671830033185,
        "timeout": false
      }
    },
    "before_public": {
      "passed": false,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n/work/Program.cs(12,45): warning CS0649: Field 'Subject.ManualClassification.X' is never assigned to, and will always have its default value 0 [/work/Study.csproj]\n/work/Program.cs(12,62): warning CS0649: Field 'Subject.ManualClassification.Letter' is never assigned to, and will always have its default value null [/work/Study.csproj]\n\nBuild succeeded.\n\n/work/Program.cs(12,45): warning CS0649: Field 'Subject.ManualClassification.X' is never assigned to, and will always have its default value 0 [/work/Study.csproj]\n/work/Program.cs(12,62): warning CS0649: Field 'Subject.ManualClassification.Letter' is never assigned to, and will always have its default value null [/work/Study.csproj]\n    2 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.72\n",
        "output_truncated": false,
        "elapsed_seconds": 3.2198127029987518,
        "timeout": false
      },
      "run": {
        "exit_code": 1,
        "output": "NotImplementedException: The method or operation is not implemented.\n",
        "output_truncated": false,
        "elapsed_seconds": 0.31465329099592054,
        "timeout": false
      }
    },
    "eligible": true,
    "fixture_sha256": "7582e1a64082d4ee1423c6a657c93c5325d5c200617a1fb87e13101b36b1ea97"
  },
  {
    "case": "dependencies-to-project",
    "reference_public": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.67\n",
        "output_truncated": false,
        "elapsed_seconds": 3.1192520179974963,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "Changing App's dependency on Lib from a package reference to a project reference\nCHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.31447311200463446,
        "timeout": false
      }
    },
    "reference_hidden": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.70\n",
        "output_truncated": false,
        "elapsed_seconds": 3.169474397000158,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.3144486860037432,
        "timeout": false
      }
    },
    "before_public": {
      "passed": false,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.68\n",
        "output_truncated": false,
        "elapsed_seconds": 3.1187596719973953,
        "timeout": false
      },
      "run": {
        "exit_code": 1,
        "output": "Exception: Assertion failed\n",
        "output_truncated": false,
        "elapsed_seconds": 0.31441450100101065,
        "timeout": false
      }
    },
    "eligible": true,
    "fixture_sha256": "ca1099a1c8db2d0bb88e84eddc9c1d6d96dd72dc8b155759f1331a542e1bd15a"
  },
  {
    "case": "dependencies-to-package",
    "reference_public": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.65\n",
        "output_truncated": false,
        "elapsed_seconds": 3.1201088929956313,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "Changing App's dependency on Lib from a project reference to a package reference\nCHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.314660254996852,
        "timeout": false
      }
    },
    "reference_hidden": {
      "passed": true,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.68\n",
        "output_truncated": false,
        "elapsed_seconds": 3.1698894080036553,
        "timeout": false
      },
      "run": {
        "exit_code": 0,
        "output": "CHECKS_PASSED\n",
        "output_truncated": false,
        "elapsed_seconds": 0.3145520010002656,
        "timeout": false
      }
    },
    "before_public": {
      "passed": false,
      "stage": "execute",
      "build": {
        "exit_code": 0,
        "output": "An issue was encountered verifying workloads. For more information, run \"dotnet workload update\".\n\nBuild succeeded.\n    0 Warning(s)\n    0 Error(s)\n\nTime Elapsed 00:00:02.66\n",
        "output_truncated": false,
        "elapsed_seconds": 3.119711915998778,
        "timeout": false
      },
      "run": {
        "exit_code": 1,
        "output": "Exception: Assertion failed\n",
        "output_truncated": false,
        "elapsed_seconds": 0.314542692998657,
        "timeout": false
      }
    },
    "eligible": true,
    "fixture_sha256": "71c4afc3b117f9f0754bb55dd818a909846ae220e83a780566d0ab917325f519"
  }
]
</build:entry>

<build:entry path="docs/developers/historical-code-benchmark.md">
# Compare local models with historical nloum code

For maintainers running a small personal coding benchmark and inspecting readable model context.

This first curated dataset contains eight C# tasks from three `nloum` repositories: two reference-conversion repairs, an exception-factory refactor, and five method completions. Both author and committer dates precede 1 January 2021. Each record retains the commit, parent when present, original source location, raw Git blob hash, extracted method hash, prompt, broken/missing method, supplied collaborators, public example and held-out checks.

The prompts were reconstructed now from inspected behavior and commits; they are not recovered historical requests. Completion tasks replace a committed body with `NotImplementedException`; repair/refactor tasks use the historical parent method. Reference methods retain their source text with normalized newlines and are compiled in reduced fixtures. This is not a full build of the original applications. A supplied helper in the variable-name case comes from a later pre-2021 snapshot instead of restoring Humanizer; that substitution is explicit in the record.

## Qualify before asking a model

A historical reference must pass public and held-out native C# checks, and the broken/missing baseline must fail the public check. The stored receipts are bound to the exact three compiled fixture sources. Reference success is therefore an inclusion criterion, not an unbiased estimate that the human solved 100% of arbitrary tasks. The checks were authored for this study after inspecting the changes and may miss behavior outside their stated contract.

The SDK and runtime images must already be installed. Candidate code builds in a network-disabled Docker container and executes in a second restricted, read-only container that receives binaries only. There is no workspace, credential or Docker-socket mount. Time and memory are bounded, and timed-out containers are removed. This is process confinement and finite behavioral testing, not proof of semantic correctness or adversarial test integrity.

```sh
PYTHONPATH=apps/agent/src apps/agent/.venv/bin/python -m hickory_agent_worker.history_benchmark \
  --dataset docs/experiments/historical-code/2026-11-01/dataset.json \
  --references target/local-agent/history/fresh-references.json --verify
```

## Read the task in context

Requests contain prose and fenced code, not a JSON dump. The current method is followed immediately by the actual public-check failure, then read-only collaborators and a public example. Each repair request contains the original task, current candidate and current public feedback. Earlier attempts remain in the journal. Requests ask for a cohesive document with exactly one C# method block. Malformed replies are failed attempts; explanation quality is not independently graded.

Only whitelisted fields enter the request. The target reference answer, commit messages and held-out check bodies remain outside the model context. Some read-only collaborator methods also appear as targets in other cases; this is deliberate supplied context and makes the cases dependent. Public compilation never includes held-out check bodies, so compiler errors cannot disclose them. A truncated response is never compiled or executed. At public success, repair stops; held-out checks run once afterward and their feedback cannot guide another attempt.

JSON is retained for machine-readable artifacts and measurements. `--layout separate` moves the same human-readable blocks into a separate evidence section. The initial coding trial uses `literate`; it does not by itself estimate a layout effect or exercise the live Hickory action registry.

```sh
PYTHONPATH=apps/agent/src apps/agent/.venv/bin/python -m hickory_agent_worker.history_benchmark \
  --dataset docs/experiments/historical-code/2026-11-01/dataset.json \
  --references target/local-agent/history/fresh-references.json \
  --models qwen3-coder:30b qwen3.8:27b --iterations 2 --layout literate \
  --output target/local-agent/history/new-trial
```

Both models receive CPU-only execution, six threads, 8192-token context, 1536-token response limit, temperature 0.2, top_p 0.95, top_k 20, seed 42 and thinking disabled. Requests use native Ollama text output. Each task allows at most two attempts with public feedback. Exact requests, replies, timing phases, candidate code, failures, model digests and image identities are retained. Models run sequentially and unload between models. This is a fixed-budget comparison rather than per-model sampling optimization.

## Interpret the comparison

Compare model public/held-out success and first-attempt versus repaired success against the qualified historical method. Distinct correct implementations pass; textual similarity to the historical code is not the correctness score. Report elapsed time and tokens for model runs, but do not infer human work time from the spacing between commits. These histories contain no reliable measurement of time spent, false starts or original assistance.

There are eight tasks but only four source commits, including multiple closely related tasks. Do not treat them as eight independent samples of a programmer's ability. The tiny curated set, retrospective specifications, supplied collaborators and limited tests cannot establish general parity with the author. Public pre-2021 code may be in model training data; its age does not establish contamination freedom. No tuning split is established here, and further optimization against these cases would turn them into development data.

[Dataset and reference methods](../experiments/historical-code/2026-11-01/dataset.json), [source provenance and exclusions](../experiments/historical-code/2026-11-01/provenance.json), and [qualified reference receipts](../experiments/historical-code/2026-11-01/references.json).

## Experiment with debugger probes between model calls

This is a proposed extension, not an implemented batch debugger. The existing native bridge waits for a step to settle and records source/stop/frame-scoped expression receipts. The UI step action acknowledges dispatch before the stop arrives. An autonomous sequence must use a settled-stop acknowledgement and refresh scope between operations, rather than issue repeated UI clicks against one old handle.

Existing watches refresh at pauses, and adapter capabilities expose conditional breakpoints, hit conditions and logpoints. First test whether a model can install a few targeted probes, continue until an interesting stop and read one source-aligned trace. The primary hypothesis is that coherent source-aligned observations improve diagnosis and verified repairs; reducing repeated inference is secondary. Where stepping is necessary, allow a bounded plan: evaluate chosen expressions, step over at most N stops, collect value changes, and stop on a requested predicate, breakpoint, exception, source/frame change, cancellation or budget exhaustion. Each operation must retain the existing session permissions and freshness checks. Current readings bind frame handles to a stop; refresh them after stepping rather than reuse the initial handle. Following a call across stops needs an explicit continuity policy, and ambiguous continuity should stop the plan. Reverse steps require actual adapter support; browsing recorded earlier values does not reverse the running program.

Represent the plan and observations as sections in a literate document. Group expression changes and messages at the lines and stops that produced them; retain the full ordered receipts separately. A compact reading can show where an invariant first failed without making the model process an unrelated transcript or a JSON state dump.

First replay identical debugger receipts as source-aligned versus chronological readings, keeping facts and available actions fixed. Make verified repair success the primary outcome; count wrong-source and wrong-frame associations as diagnostic outcomes. Then compare one-model-call-per-step execution, fixed batches and bounded conditional probes in a separate interactive experiment with equal permitted operations and runtime budgets. Adaptive probes may collect different facts, so their effect cannot be attributed to placement alone. Report calls, tokens and elapsed time as secondary measurements. The historical coding trial above uses compiler/public-check feedback; it does not yet measure this debugger hypothesis. See the [factorial study protocol](../../.hickory-source/mechanisms/apps/agent/factorial-context-study.md) for feature interactions and confirmation requirements.
</build:entry>

</build:doc>
