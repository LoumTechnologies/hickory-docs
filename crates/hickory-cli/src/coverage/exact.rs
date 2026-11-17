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
