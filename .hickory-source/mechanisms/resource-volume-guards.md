<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep declared output folders out of resource publication

For maintainers checking create/move targets against saved, dirty and candidate ownership.

Compiler output inventories name produced files. A declared volume also reserves
its output folder, including names not produced yet. Treating that folder as an
ordinary source destination permits a provider to publish an unowned generated
file. A real preparation reproduced this for a dirty `output="new-out"` declaration
and for a candidate changing the declaration while creating `new-out/fresh.ts`.
Neither preparation changed source; both must become explicit refusals.

## Interpret only complete, bound declarations

Use the parsed source bodies already retained by the ownership scan. Do not read
another revision or promote a declared folder into observed compiler output.
Normalize relative folders against the declaring document, retaining literal UTF-8
names. Reuse the held-source path policy to refuse symlinked or differently spelled
physical folder aliases before matching. Exact folder/ancestor membership avoids
similarly prefixed siblings. An
ingested volume no longer flushes its directory, matching the existing listing
policy. Dynamic, absolute or escaping output scopes remain unknown here and refuse
guarded resource work; readable source remains independent of this capability.

<ui:copy id="resource-declared-volume-claims">
pub(crate) fn declared_volume_targets(
    root: &Path,
    sources: &BTreeMap<String, String>,
    targets: &[String],
) -> Result<BTreeSet<String>, String> {
    let mut folders = BTreeSet::new();
    for (path, source) in sources {
        let document = hick_lang::parse(source).map_err(|error| {
            format!("Output-folder ownership cannot be parsed: {path}: {error}")
        })?;
        let tags = document.all_tags();
        if tags.iter().any(|tag| tag.name == "ingested") {
            continue;
        }
        for tag in tags.into_iter().filter(|tag| tag.name == "volume") {
            let Some(output) = tag.get_attribute("output") else {
                continue;
            };
            let output = output.trim();
            if output.is_empty()
                || output.contains("{{")
                || output.contains("}}")
                || output.contains('\\')
                || output.contains('\0')
                || Path::new(output).is_absolute()
                || output.as_bytes().get(1) == Some(&b':')
            {
                return Err(format!(
                    "Output-folder ownership is unresolved at {path}. Preserve the source and select a literal relative output scope before resource publication."
                ));
            }
            let mut parts: Vec<&str> = path
                .rsplit_once('/')
                .map(|(parent, _)| parent.split('/').collect())
                .unwrap_or_default();
            for part in output.split('/') {
                match part {
                    "" | "." => {}
                    ".." => {
                        if parts.pop().is_none() {
                            return Err(format!(
                                "Output-folder ownership escapes this workspace at {path}. No source resource was changed."
                            ));
                        }
                    }
                    name => parts.push(name),
                }
            }
            folders.insert(parts.join("/"));
            if folders.len() > 10000 {
                return Err("Output-folder ownership exceeds the interactive review budget. Narrow the construction scope.".into());
            }
        }
    }
    let mut names = Names::default();
    for folder in &folders {
        if !folder.is_empty() {
            confined(root, folder, &mut names)?;
        }
    }
    let mut protected = BTreeSet::new();
    for target in targets {
        let mut current = target.as_str();
        loop {
            if folders.contains(current) || folders.contains("") {
                protected.insert(target.clone());
                break;
            }
            match current.rsplit_once('/') {
                Some((parent, _)) => current = parent,
                None => break,
            }
        }
    }
    Ok(protected)
}
</ui:copy>

## Check folder identity without broadening the output evidence

These focused cases cover nested declaring documents, parent traversal within the
workspace, root output folders, similarly named siblings, ingestion and unresolved
scopes. They do not publish files or execute a volume.

<ui:copy id="resource-declared-volume-witnesses">
#[cfg(test)]
mod declared_volume_tests {
    use super::*;
    fn sources(output: &str) -> BTreeMap<String, String> {
        BTreeMap::from([(
            "sub/owner.md".into(),
            format!(
                "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:volume output=\"{output}\"/></h:doc>"
            ),
        )])
    }
    #[test]
    fn exact_declared_folder_membership_is_separate_from_produced_files() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let targets = vec![
            "owned/a.ts".into(),
            "owned/deep/b.ts".into(),
            "owned-sibling/a.ts".into(),
        ];
        assert_eq!(
            declared_volume_targets(root, &sources("../owned"), &targets).unwrap(),
            BTreeSet::from([targets[0].clone(), targets[1].clone()])
        );
        assert_eq!(
            declared_volume_targets(root, &sources(".."), &targets)
                .unwrap()
                .len(),
            3
        );
        let mut ingested = sources("../owned");
        let text = ingested.get_mut("sub/owner.md").unwrap();
        *text = text.replace("</h:doc>", "<h:ingested/></h:doc>");
        assert!(
            declared_volume_targets(root, &ingested, &targets)
                .unwrap()
                .is_empty()
        );
    }
    #[cfg(unix)]
    #[test]
    fn physical_folder_aliases_cannot_hide_output_ownership() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        std::fs::create_dir(root.join("owned")).unwrap();
        std::os::unix::fs::symlink(root.join("owned"), root.join("alias")).unwrap();
        let targets = vec!["owned/fresh.ts".into()];
        assert!(declared_volume_targets(root, &sources("../alias"), &targets).is_err());
        if root.join("OWNED").is_dir() {
            assert!(declared_volume_targets(root, &sources("../OWNED"), &targets).is_err());
        }
    }
    #[test]
    fn unknown_or_external_folder_claims_never_become_write_authority() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let targets = vec!["source.ts".into()];
        for output in [
            "{{destination}}",
            "../../escape",
            "/absolute",
            "C:/absolute",
            "",
            "back\\slash",
        ] {
            assert!(
                declared_volume_targets(root, &sources(output), &targets).is_err(),
                "{output}"
            );
        }
    }
}
</ui:copy>
## Repeat ownership admission through the real service

The HTTP witness captures a physical source guard before retaining a dirty volume
change. It checks both old and new folder reservations, a similarly named ordinary
sibling, a caller-forged creation capability and a candidate moving the folder.
The explicit save must refuse before source or recovery publication. Declared
folders never appear as observed compiler output paths.

<ui:copy id="resource-volume-guard-http-driver">
#!/usr/bin/env python3
"""Actual saved/draft/candidate output-folder refusals, with no source publication."""
import copy,importlib.util,json,os,tempfile,urllib.error,urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-volume-guards-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();entry='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="part.md"/></h:doc>';part='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:volume name="generated" output="old-out"/><h:file path="existing.py">x</h:file></h:doc>';(root/'project.md').write_text(entry);(root/'part.md').write_text(part)
  for name in ['old-out','new-out','new-out-sibling']:(root/name).mkdir()
  engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
  try:
   origin=fixture.origin(engine)
   def get(route):
    with urllib.request.urlopen(origin+'/api/'+route,timeout=30) as reply:return json.load(reply)
   def call(route,body):
    req=urllib.request.Request(origin+'/api/changes/'+route,data=json.dumps(body).encode(),headers={'Content-Type':'application/json'},method='POST')
    with urllib.request.urlopen(req,timeout=30) as reply:return json.load(reply)
   def refused(route,body,status=409,contains=None):
    try:call(route,body)
    except urllib.error.HTTPError as error:
     detail=error.read().decode();assert error.code==status,(error.code,detail)
     if contains:assert contains in detail,detail
    else:raise AssertionError('A protected output-folder intention was admitted: '+route)
   node=next(row for row in get('files')['tree'] if row['path']=='part.md');guard=get('docs/'+node['doc_id'])['save_guard']
   names=['old-out/fresh.ts','new-out/fresh.ts','new-out-sibling/fresh.ts'];edit={'documentChanges':[{'kind':'create','uri':'hick:///'+name} for name in names]};capture={'context':'dirty-prefix','ownership_sources':{'part.md':part.replace('old-out','new-out')},'saved_guards':{'part.md':guard},'edit':edit};inputs=call('resource-inputs',capture)
   for name in names[:2]:assert inputs['sources'][name]=={'kind':'missing','creatable':False}
   assert inputs['sources'][names[2]]=={'kind':'missing','creatable':True}
   forged=copy.deepcopy(inputs)
   for name in names[:2]:forged['sources'][name]['creatable']=True
   plan=call('plan-resources',{'context':capture['context'],'title':'Forged permissions','edit':edit,'sources':forged['sources']})['plan'];refused('resource-consequences',{'inputs':forged,'capture':capture,'plan':plan},409)
   changes={'documentChanges':[{'textDocument':{'uri':'hick:///part.md','version':None},'edits':[{'range':{'start':{'line':0,'character':0},'end':{'line':0,'character':len(part)}},'newText':part.replace('old-out','new-out')}]},{'kind':'create','uri':'hick:///new-out/fresh.ts'}]};capture={'context':'candidate-prefix','edit':changes};inputs=call('resource-inputs',capture);plan=call('plan-resources',{'context':capture['context'],'title':'Candidate folder guard','edit':changes,'sources':inputs['sources']})['plan'];facts=call('resource-consequences',{'capture':capture,'inputs':inputs,'plan':plan});assert facts['protected_targets']==['new-out/fresh.ts'];assert not facts['source_files_modified']
   for layer in ['saved','draft']:assert 'new-out/fresh.ts' not in facts[layer]['output_paths'],'A folder declaration was presented as produced output'
   refused('resource-save',{'request_id':'d'*32,'reviewed_fingerprint':facts['fingerprint'],'proposal':{'capture':capture,'inputs':inputs,'plan':plan}},contains='generated')
   assert get('changes/resource-recoveries')['ids']==[]
   for name in names:assert not(root/name).exists()
   assert(root/'part.md').read_text()==part and(root/'project.md').read_text()==entry
   print('PASS: saved and dirty folder claims, candidate folder change, forged creation permission and explicit publication refusal; no output materialization or source/recovery publication. Folder claims remain separate from observed compiler outputs.')
  finally:fixture.stop(engine)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-resource-volume-guards.py"><ui:paste select="#resource-volume-guard-http-driver"/></ui:file>
</ui:doc>
