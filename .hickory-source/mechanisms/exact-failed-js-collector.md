<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Select JavaScript tests without widening native file scope

For maintainers tracing an editor Run or retained failure to the native runner selection, source authority and measurement.

A framework name identifies a case only within its source file and runner project. A global union of failed names and files creates a Cartesian selection, so two files with crossed failures execute passing cases too. This adapter runs one exact file group at a time and merges the producer's Istanbul maps. It retains each native result suite before retiring a changed project or checking that the executed pairs exactly match the requested failures. It compares the native regular expression against every previously observed name before running any group. Jest compiles name patterns case-insensitively. When the runtime supports scoped flag modifiers, disable that flag inside the exact expression. Probe the runtime first; on older runtimes, a case-only competitor refuses before bodies. Escaped metacharacters stay literal. It refuses multiple native projects or ambiguous same-file names; it does not invent harness identity.

Jest provides exact `runTestsByPath`. Vitest positional filters match substrings, so its public specification API selects one exact real file before running any bodies. Vitest collection uses its public no-body API and the same native JSON reporter as execution. Its ordinary list command emits presentation names with ` > ` separators, which cannot be parsed back into literal suite/test names. A framed internal JSON result keeps producer hierarchy names intact even when diagnostics contain braces. Reporter/cache files stay under the existing Rust-held artifact root, whose identity is checked before the observation is adopted. Configuration remains the project's native configuration. The same driver serves source-adjacent terminal runs: preserve the explicitly labelled name-filter family, select one exact native file, keep filter whitespace, and require the same captured saved bytes before bodies. Empty filter means only an explicitly requested containing-file scope; an empty native test name remains expressible for exact retained reruns. Producer libraries resolve through their own package dependencies, including isolated package layouts. Coverage and result exports use the original private artifact parent; the Rust collector still checks its held directory identity and reads reports without following links. Arbitrary project/test code remains trusted code, not a sandbox.

<ui:copy id="exact-failed-js-native-driver">
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
</ui:copy>

Native source/hash/physical-file checks cover every requested group before any body, then repeat at each group. Retain each returned native suite before checking whether the root retired; a later refusal keeps observed outcomes while preventing the next group. A terminal source run uses the same exact file mechanism without inventing per-case verification from its exit code.

The maintained provider replay uses identical full names in two files and opposite failing bodies. It requires four unique retained case identities, two executed cases in the rerun, a genuine merged report and two retained sessions. A single-file replay also checks skipped native rows stay visible. These are service-level runner checks; physical input and accessibility acceptance remain separate.

Native names remain readable labels. Retained JS identities instead hash the versioned provider/source/name tuple, so identical display names in separate files cannot collapse into one case. Failed-only is a new execution request, so ordinary fixes must not require restoring the old source. The original record must still name the same physical workspace. Every rerun acquires a no-body native listing against current captured bytes, so an earlier partial harness listing cannot authorize broader execution. Reconcile only the original failed identities. Keep the old evidence and its source hashes unchanged; a separate current selection supplies new coordinates and guards. Missing or ambiguous current cases refuse before bodies. The private `FailedSelection` contains current native cases and the old failed ID set; it never carries old coverage as current authority. Native discovery receives the original held root lease, and any source change after capture invalidates rather than renews that selection. Legacy JS records without this qualification remain readable but cannot authorize an exact rerun. Duplicate unqualified native names in Rust or .NET also refuse instead of broadening selection.

<ui:copy id="exact-failed-runner-plan">
//! Exact retained identities and newly captured failed-only selections.
use super::*;
use sha2::{Digest,Sha256};
pub(super) fn qualify(provider:&str,tests:&mut[TestResult]) {
 if !["jest","vitest"].contains(&provider){return;}
 for test in tests {let name=test.runner_name.get_or_insert_with(||test.id.clone()).clone();if let Some(path)=&test.path{let tuple=serde_json::to_vec(&("js-case-v1",provider,path,&name)).unwrap();test.id=format!("js-v1:{:x}",Sha256::digest(tuple));}}
}
/// Current native cases are selection inputs, never rewritten historical evidence.
#[derive(Debug)]
struct FailedSelection {provider:String,cases:Vec<TestResult>,wanted:std::collections::BTreeSet<String>}
fn select(previous:&Record,cases:Vec<TestResult>)->Result<FailedSelection>{
 let wanted=previous.tests.iter().filter(|case|case.status=="failed").map(|case|case.id.clone()).collect::<std::collections::BTreeSet<_>>();ensure!(!wanted.is_empty(),"No failed tests with runner identities");
 let old_ids=previous.tests.iter().map(|case|&case.id).collect::<std::collections::BTreeSet<_>>();let current_ids=cases.iter().map(|case|case.id.clone()).collect::<std::collections::BTreeSet<_>>();
 ensure!(old_ids.len()==previous.tests.len()&&current_ids.len()==cases.len(),"Repeated native test names lack a unique source/harness identity; exact failed-only rerun is unavailable");
 ensure!(wanted.iter().all(|id|current_ids.contains(id)),"A previously failed case no longer has one exact current runner identity; run the current suite instead");
 if ["jest","vitest"].contains(&previous.identity.collector.as_str()){ensure!(previous.tests.iter().chain(cases.iter()).all(|case|case.id.starts_with("js-v1:")&&case.runner_name.is_some()&&case.path.is_some()),"Legacy or unlocated JavaScript cases lack exact source-bound identity; run the current project first");}
 Ok(FailedSelection{provider:previous.identity.collector.clone(),cases,wanted})
}
fn same_workspace(previous:&Record,current:&Record)->Result<()>{ensure!(previous.workspace_identity_kind.as_deref()==Some("physical-root-v1")&&previous.workspace_id==current.workspace_id&&previous.workspace_identity_kind==current.workspace_identity_kind&&previous.project==current.project&&previous.language==current.language,"Original failed-test physical workspace/project is unavailable");Ok(())}
/// Same-source entry used by focused policy tests; changed source requires prepare's fresh query.
#[cfg(test)]
pub(super) fn plan(root:&Path,project:&Path,current:&mut Record,previous:&Record)->Result<commands::Plan>{same_workspace(previous,current)?;ensure!(!previous.source_changed_during_run&&previous.identity.source_hashes==current.identity.source_hashes,"Changed failed-test source requires fresh runner discovery");let selection=select(previous,previous.tests.clone())?;build(root,project,current,&selection)}
pub(super) async fn prepare(root:&Path,project:&Path,current:&mut Record,previous:&Record)->Result<commands::Plan>{
 let scope=crate::serve::resource_inputs::RootIdentity::bind(root);scope.check(root).map_err(anyhow::Error::msg)?;same_workspace(previous,current)?;
 ensure!(current.workspace_id==crate::serve::resource_inputs::RootIdentity::fingerprint(root).ok(),"Original failed-test workspace changed before selection");
 let provider=match current.language.as_str(){"python"=>"pytest-cov","rust"=>"cargo-llvm-cov","csharp"=>"coverlet","typescript"|"javascript"|"tsx"|"jsx"|"typescriptreact"|"javascriptreact"=>commands::js_runner(project)?,_=>bail!("No supported native provider for this failed-test selection")};ensure!(provider==previous.identity.collector,"Native test provider changed since the failed run");
 let result={
  let mut captured=None;
  for _ in 0..3{
   scope.check(root).map_err(anyhow::Error::msg)?;
   let observation=discovery::inspect_scoped(root,project,&current.language,&scope).await?;
   scope.check(root).map_err(anyhow::Error::msg)?;
   ensure!(observation["version"]==1&&observation["workspace_id"].as_str()==current.workspace_id.as_deref()&&observation["project"]==current.project&&observation["language"]==current.language,"Fresh runner discovery belongs to another selection scope");
   if observation["source_state"]!="current"{continue;}
   let sources:BTreeMap<String,String>=serde_json::from_value(observation["source_hashes"].clone())?;
   let metadata=config(project)?;if snapshot(root)?!=sources{continue;}
   let cases:Vec<TestResult>=serde_json::from_value(observation["tests"].clone())?;let selection=select(previous,cases)?;
   if current.language=="csharp"{
    let filtered=discovery::inspect_filtered_csharp(root,project,&selection.wanted.iter().cloned().collect::<Vec<_>>(),&scope).await?;scope.check(root).map_err(anyhow::Error::msg)?;
    ensure!(filtered["version"]==1&&filtered["workspace_id"].as_str()==current.workspace_id.as_deref()&&filtered["project"]==current.project,"Native filtered discovery belongs to another scope");
    if filtered["source_state"]!="current"||serde_json::from_value::<BTreeMap<String,String>>(filtered["source_hashes"].clone())?!=sources{continue;}
    let matches:Vec<TestResult>=serde_json::from_value(filtered["tests"].clone())?;let ids=matches.iter().map(|case|case.id.clone()).collect::<std::collections::BTreeSet<_>>();ensure!(ids.len()==matches.len()&&ids==selection.wanted,"Native .NET filter cannot express these exact failed identities without additional cases; run the current suite instead");
   }

   current.identity.source_hashes=sources;current.metadata=metadata;
   current.notes=format!("Failed-only rerun of {}. Current native cases captured by runner discovery {}. Original evidence is retained independently.",previous.id,observation["fingerprint"].as_str().unwrap_or("unavailable"));
   captured=Some(build(root,project,current,&selection)?);break;
  }
  captured.context("Current runner cases did not stabilize against saved source; refresh and retry")?
 };
 scope.check(root).map_err(anyhow::Error::msg)?;ensure!(snapshot(root)?==current.identity.source_hashes,"Saved source changed after failed-case selection; refresh and retry");Ok(result)
}
fn build(root:&Path,project:&Path,record:&mut Record,selection:&FailedSelection)->Result<commands::Plan>{
 if !["jest","vitest"].contains(&selection.provider.as_str()){return commands::automated(root,project,record,&selection.wanted.iter().cloned().collect::<Vec<_>>());}
 let mut groups:BTreeMap<String,Vec<String>>=BTreeMap::new();
 for test in selection.cases.iter().filter(|case|selection.wanted.contains(&case.id)){
  let file=test.path.as_ref().unwrap();let name=test.runner_name.as_ref().unwrap();let mut proof=vec![test.clone()];qualify(&selection.provider,&mut proof);ensure!(proof[0].id==test.id,"JavaScript native case identity is inconsistent");groups.entry(file.clone()).or_default().push(name.clone());
 }
 ensure!(groups.len()<=128,"Exact rerun exceeds 128 source groups");let mut native=Vec::new();
 for(file,names)in groups{let path=safe_path(root,&file)?;ensure!(path.is_file()&&path.starts_with(project),"Failed test source is outside its original project");ensure!(names.len()<=1024,"Exact rerun exceeds 1024 cases in one source file");let hash=record.identity.source_hashes.get(&file).context("Failed test source has no captured preimage")?;let known_names=selection.cases.iter().filter(|case|case.path.as_deref()==Some(file.as_str())).map(|case|case.runner_name.as_ref().unwrap()).collect::<Vec<_>>();native.push(json!({"file":path,"hash":hash,"names":names,"known_names":known_names}));}
 let mut plan=commands::automated(root,project,record,&[])?;ensure!(record.identity.collector==selection.provider,"Native test provider changed since failed-case selection");let artifacts=plan.artifacts.as_ref().context("Private exact collector artifacts required")?;
 #[cfg(unix)]let(project_dev,project_ino)={use std::os::unix::fs::MetadataExt;let metadata=std::fs::metadata(project)?;(metadata.dev().to_string(),metadata.ino().to_string())};
 #[cfg(not(unix))]let(project_dev,project_ino):(String,String)={bail!("Exact JavaScript physical project binding is unavailable on this host")};
 let spec=json!({"project":project,"project_dev":project_dev,"project_ino":project_ino,"provider":selection.provider,"groups":native,"artifacts":artifacts.path(),"reports":plan.report.parent().unwrap(),"result":plan.result});plan.argv=vec!["node".into(),"--input-type=module".into(),"--eval".into(),hick_plugins::testing::JAVASCRIPT_TEST_DRIVER.into(),spec.to_string()];plan.exact_results=true;record.argv=plan.argv.clone();Ok(plan)
}
#[cfg(test)]mod tests{
 use super::*;
 fn case(path:&str,name:&str,status:&str)->TestResult{serde_json::from_value(json!({"id":name,"status":status,"output":"","path":path,"line":null,"labels":[]})).unwrap()}
 #[test]fn native_names_in_distinct_sources_have_stable_distinct_identities(){let mut tests=vec![case("a.test.js","cohort negative","failed"),case("b.test.js","cohort negative","passed")];qualify("jest",&mut tests);assert_ne!(tests[0].id,tests[1].id);assert_eq!(tests[0].runner_name.as_deref(),Some("cohort negative"));let old=tests.clone();qualify("jest",&mut tests);assert_eq!(tests[0].id,old[0].id);let mut different=old.clone();qualify("vitest",&mut different);assert_ne!(different[0].id,old[0].id);}
 #[test]fn exact_rerun_preserves_source_name_pairs_and_refuses_ambiguous_native_names(){let dir=tempfile::tempdir().unwrap();let root=dir.path().canonicalize().unwrap();std::fs::write(root.join("package.json"),r#"{"devDependencies":{"jest":"29.7.0"}}"#).unwrap();for file in ["a.test.js","b.test.js"]{std::fs::write(root.join(file),"// source").unwrap();}let mut prior=store::new_record(&root,&root,&Request{language:"javascript".into(),action:"run".into(),..Default::default()}).unwrap();prior.identity.collector="jest".into();prior.tests=vec![case("a.test.js","cohort negative","failed"),case("b.test.js","cohort positive","failed")];qualify("jest",&mut prior.tests);let mut next=prior.clone();let plan=plan(&root,&root,&mut next,&prior).unwrap();let spec:Value=serde_json::from_str(plan.argv.last().unwrap()).unwrap();assert!(plan.exact_results);assert_eq!(spec["groups"][0]["names"],json!(["cohort negative"]));assert_eq!(spec["groups"][1]["names"],json!(["cohort positive"]));let mut duplicate=prior.clone();duplicate.tests.push(duplicate.tests[0].clone());assert!(super::plan(&root,&root,&mut next,&duplicate).unwrap_err().to_string().contains("harness"));let mut stale=prior.clone();stale.identity.source_hashes.insert("a.test.js".into(),"changed".into());assert!(super::plan(&root,&root,&mut next,&stale).is_err());for provider in ["cargo-llvm-cov","coverlet"]{let mut unknown=prior.clone();unknown.identity.collector=provider.into();unknown.tests=vec![case("a.test.js","same","failed"),case("b.test.js","same","passed")];assert!(super::plan(&root,&root,&mut next,&unknown).unwrap_err().to_string().contains("harness"));}}
 #[test]fn ambiguous_declared_collectors_refuse_instead_of_choosing_one(){let dir=tempfile::tempdir().unwrap();std::fs::write(dir.path().join("package.json"),r#"{"devDependencies":{"vitest":"3.2.7","jest":"29.7.0"}}"#).unwrap();assert!(commands::js_runner(dir.path()).unwrap_err().to_string().contains("unambiguous"));}

 #[test]fn fresh_current_case_membership_does_not_rewrite_historical_outcomes(){let temp=tempfile::tempdir().unwrap();let root=temp.path().canonicalize().unwrap();std::fs::write(root.join("package.json"),r#"{"devDependencies":{"jest":"29.7.0"}}"#).unwrap();std::fs::write(root.join("a.test.js"),"// original").unwrap();let mut prior=store::new_record(&root,&root,&Request{language:"javascript".into(),action:"run".into(),..Default::default()}).unwrap();prior.identity.collector="jest".into();prior.tests=vec![case("a.test.js","cohort negative","failed"),case("a.test.js","cohort positive","passed")];qualify("jest",&mut prior.tests);let sources=prior.identity.source_hashes.clone();let mut fresh=prior.tests.clone();fresh[0].line=Some(20);fresh[0].status="discovered".into();let captured=select(&prior,fresh.clone()).unwrap();assert_eq!(captured.cases[0].line,Some(20));assert_eq!(prior.tests[0].line,None);assert_eq!(prior.identity.source_hashes,sources);assert_eq!(captured.wanted.len(),1);fresh.remove(0);assert!(select(&prior,fresh).unwrap_err().to_string().contains("no longer"));let mut duplicate=prior.tests.clone();duplicate.push(duplicate[0].clone());assert!(select(&prior,duplicate).unwrap_err().to_string().contains("harness"));}

}
</ui:copy>

<ui:file path="crates/hickory-cli/src/coverage/exact.rs"><ui:paste select="#exact-failed-runner-plan"/></ui:file>
<ui:file path="crates/hick-plugins/src/testing/javascript.mjs"><ui:paste select="#exact-failed-js-native-driver"/></ui:file>
</ui:doc>
