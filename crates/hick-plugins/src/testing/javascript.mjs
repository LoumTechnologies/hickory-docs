import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {spawnSync} from 'node:child_process';
const spec=JSON.parse(process.argv.at(-1));
const require=createRequire(path.join(spec.project,'package.json'));
const original={dev:spec.project_dev,ino:spec.project_ino};
function assertProject(){const current=fs.statSync(spec.project,{bigint:true}),cwd=fs.statSync(process.cwd(),{bigint:true});if(fs.realpathSync(spec.project)!==spec.project||String(current.dev)!==original.dev||String(current.ino)!==original.ino||cwd.dev!==current.dev||cwd.ino!==current.ino)throw Error('Original collector project was replaced; no further native test group was started');}
function source(file,hash){assertProject();if(fs.realpathSync(file)!==file||crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex')!==hash)throw Error('Original selected test source changed');return file;}
function scopedPattern(pattern){try{const probe=new RegExp('(?-i:^Case$)','i');if(probe.test('Case')&&!probe.test('case'))return '(?-i:'+pattern+')';}catch{}return null;}
function exactPattern(names,known){const escaped=names.map(name=>name.replace(/[.*+?^${}()|[\]\\]/g,'\\$&'));let pattern='^(?:'+escaped.join('|')+')(?![\\s\\S])';if(spec.provider==='jest')pattern=scopedPattern(pattern)??pattern;const native=new RegExp(pattern,spec.provider==='jest'?'i':'');if(names.some(name=>!known.includes(name))||known.some(name=>native.test(name)!==names.includes(name)))throw Error('Native matching cannot select the exact failed names without additional known bodies');return pattern;}
function jestCore(){return createRequire(createRequire(require.resolve('jest')).resolve('@jest/core'));}
function coverageModules(){return createRequire(spec.provider==='jest'?jestCore().resolve('@jest/reporters'):require.resolve('@vitest/coverage-v8'));}
async function runNative(file,pattern,directory,coverage){
 const result=path.join(directory,'results.json');
 if(spec.provider==='jest'){
  const {configs}=await jestCore()('jest-config').readConfigs({_:[], $0:'hickory-source-selection'},[spec.project]);
  if(configs.length!==1)throw Error('Source selection requires a qualified Jest project');
  const argv=[require.resolve('jest/bin/jest'),'--runTestsByPath',file,'--cacheDirectory',path.join(directory,'jest-cache'),'--json','--outputFile',result];
  if(pattern!==undefined)argv.push('--testNamePattern',pattern);
  if(coverage)argv.push('--coverage','--coverageReporters=json','--coverageDirectory',directory);else argv.push('--coverage=false');
  assertProject();const child=spawnSync(process.execPath,argv,{cwd:spec.project,encoding:'utf8',maxBuffer:16*1024*1024});process.stdout.write(child.stdout||'');process.stderr.write(child.stderr||'');if(child.error||![0,1].includes(child.status))throw Error('Native Jest process unavailable: '+(child.error||child.status));
 }else if(spec.provider==='vitest'){
  const {createVitest}=await import(pathToFileURL(require.resolve('vitest/node')).href);
  const ctx=await createVitest('test',{watch:false,run:true,cache:false,reporters:coverage?['json']:['default','json'],outputFile:result,testNamePattern:pattern,coverage:coverage?{enabled:true,reporter:['json'],reportsDirectory:directory,clean:false,cleanOnRerun:false,reportOnFailure:true}:{enabled:false}},{cacheDir:path.join(directory,'vite-cache')});
  try{await ctx.init();assertProject();const matches=(await ctx.globTestSpecifications()).filter(item=>fs.realpathSync(item.moduleId)===file);if(matches.length!==1)throw Error('Source selection requires one qualified Vitest specification');await ctx.runTestSpecifications(matches,true);}finally{await ctx.close();}
 }else throw Error('Unsupported JavaScript test provider');
 return JSON.parse(fs.readFileSync(result,'utf8'));
}
function reportedSource(suite,file){if(path.resolve(suite.name)!==file)throw Error('Native runner reported an unrequested source file');}
assertProject();
if(spec.mode==='discover'){
 if(spec.provider!=='vitest')throw Error('Native collection reporter mode requires Vitest');
 const directory=process.env.HICKORY_COLLECTOR_ARTIFACTS;if(!directory)throw Error('Native discovery requires its held private artifact scope');const result=path.join(directory,'discovery-results.json');
 const {createVitest}=await import(pathToFileURL(require.resolve('vitest/node')).href);
 const ctx=await createVitest('test',{watch:false,run:true,cache:false,reporters:['json'],outputFile:result,coverage:{enabled:false}},{cacheDir:path.join(directory,'vite-cache')});
 try{await ctx.init();assertProject();const collected=await ctx.collectTests(await ctx.globTestSpecifications());if(collected.unhandledErrors.length)throw Error('Native Vitest collection failed');await ctx.report('onFinished',ctx.state.getFiles(),collected.unhandledErrors);assertProject();const native=JSON.parse(fs.readFileSync(result,'utf8'));process.stdout.write('HICKORY_NATIVE_DISCOVERY_V1 '+JSON.stringify(native)+'\n');}finally{await ctx.close();}
}else if(spec.mode==='source'){
 const file=source(spec.file,spec.source_hash);let pattern=spec.pattern??undefined;
 if(pattern!==undefined&&spec.provider==='jest'){pattern=scopedPattern(pattern);if(pattern===null)throw Error('This runtime cannot express case-sensitive source name filters; use runner discovery or explicitly run the containing file');}
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'hickory-source-tests-')),identity=fs.statSync(directory,{bigint:true});
 try{const native=await runNative(file,pattern,directory,false);for(const suite of native.testResults||[])reportedSource(suite,file);process.exitCode=native.success===false||native.numFailedTestSuites>0||native.numFailedTests>0?1:0;assertProject();}finally{try{const now=fs.lstatSync(directory,{bigint:true});if(!now.isSymbolicLink()&&now.dev===identity.dev&&now.ino===identity.ino)fs.rmSync(directory,{recursive:true,force:true});}catch{}}
}else{
 const groups=spec.groups;
 if(!Array.isArray(groups)||!groups.length||groups.length>128)throw Error('Bounded exact file selection required');
 const patterns=new Map(),files=new Set();
 for(const group of groups){if(typeof group.file!=='string'||files.has(group.file)||!Array.isArray(group.names)||!group.names.length||group.names.length>1024||new Set(group.names).size!==group.names.length||group.names.some(name=>typeof name!=='string'||name.length>16384)||!Array.isArray(group.known_names)||group.known_names.length>4096||group.known_names.some(name=>typeof name!=='string'||name.length>16384)||new Set(group.known_names).size!==group.known_names.length)throw Error('Unambiguous bounded source/name groups required');files.add(group.file);patterns.set(group.file,exactPattern(group.names,group.known_names));source(group.file,group.hash);}
 const producer=coverageModules(),coverage=producer('istanbul-lib-coverage').createCoverageMap({}),suites=[];let failed=false;
 const retain=complete=>fs.writeFileSync(spec.result,JSON.stringify({testResults:suites,hickory_exact_complete:complete}));
 function exportCoverage(){const context=producer('istanbul-lib-report').createContext({dir:spec.reports,coverageMap:coverage});producer('istanbul-reports').create('lcovonly').execute(context);}
 for(const [index,group]of groups.entries()){
  const file=source(group.file,group.hash),directory=path.join(spec.artifacts,'exact-'+index);fs.mkdirSync(directory);
  const native=await runNative(file,patterns.get(group.file),directory,true),rows=native.testResults||[];
  // Retain native observations before checking changed authority or complete selection.
  suites.push(...rows);retain(false);const executed=[];
  for(const suite of rows){reportedSource(suite,file);if(suite.message)process.stderr.write(suite.message+'\n');for(const test of suite.assertionResults||[]){if(['passed','failed'].includes(test.status)){executed.push(test.fullName);failed ||= test.status==='failed';}}}
  if(executed.length!==group.names.length||new Set(executed).size!==executed.length||executed.some(name=>!group.names.includes(name)))throw Error('Native runner did not execute exactly the selected failed identities; observations retained');
  coverage.merge(JSON.parse(fs.readFileSync(path.join(directory,'coverage-final.json'),'utf8')));exportCoverage();assertProject();
 }
 exportCoverage();retain(true);process.exitCode=failed?1:0;
}
