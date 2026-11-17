<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Ask a refactoring provider for edits without executing its application command

For maintainers connecting semantic refactorings to the shared source review.

Some TypeScript actions contain `_typescript.applyRefactoring`. Executing that
command is unsuitable for a preview: the provider creates destination files
before requesting client application. Instead, use its advertised
`typescript.tsserverRequest` interface for the single read operation
`getEditsForRefactor`. Retain the original provider arguments and originating
virtual file. This adapter never dispatches the application command, registers
arbitrary commands, or interprets a caller-supplied replacement action.

The protocol is documented by the [provider](https://github.com/typescript-language-server/typescript-language-server/blob/master/README.md#send-tsserver-command).
Only the held provider action can select it. Every response still passes the
existing graph revision, source acquisition, authored-range projection and
complete preview guards. Other commands remain explicitly unsupported.

## Admit one bound read and decode the complete answer

The provider's file is a physical virtual-source address, rather than an authored
Markdown address. Bind it to the queried virtual source before retaining the
read. Reject edit-plus-command combinations because ignoring either half would
present a partial action. A new file is represented as an ordered creation plus
text edit; it is never materialized by this module. TypeScript's native new-file response uses one zero-line/zero-offset insertion,
omitting `isNewFile` in `convertNewFileTextChangeToCodeEdit`. Recognize only that
complete singleton sentinel, then let shared acquisition verify that the destination
is actually missing. Mixed zero/positive positions, multiple sentinel edits and an
explicit contradictory existing-file marker refuse the entire reading. This is
protocol decoding, never permission to overwrite a destination. A malformed later member
refuses the complete response. Optional provider rename locations remain a
follow-up suggestion; this read never starts a second implicit rename or mutation.

<ui:copy id="provider-refactor-read-only-model">
use serde_json::{Value, json};
use std::collections::BTreeSet;
use tower_lsp::lsp_types::Url;

pub(crate) fn query(action: &Value, origin: &Url, advertised: bool) -> Option<Value> {
    if !advertised
        || action.get("edit").is_some_and(|v| !v.is_null())
        || action.get("disabled").is_some_and(|v| !v.is_null())
    {
        return None;
    }
    let command = action.get("command")?.as_object()?;
    if command.get("command")?.as_str()? != "_typescript.applyRefactoring" {
        return None;
    }
    let arguments = command.get("arguments")?.as_array()?;
    if arguments.len() != 1 {
        return None;
    }
    let args = arguments[0].as_object()?;
    let file = args.get("file")?.as_str()?;
    let path = if file.starts_with("file:") {
        Url::parse(file).ok()?.to_file_path().ok()?
    } else {
        std::path::PathBuf::from(file)
    };
    if path != origin.to_file_path().ok()? {
        return None;
    }
    for key in ["refactor", "action"] {
        let value = args.get(key)?.as_str()?;
        if value.is_empty() || value.len() > 16384 {
            return None;
        }
    }
    Some(
        json!({"command":"typescript.tsserverRequest","arguments":["getEditsForRefactor",arguments[0],{"expectsResult":true}]}),
    )
}

fn position(value: &Value) -> Result<Value, String> {
    let line = value
        .get("line")
        .and_then(Value::as_u64)
        .filter(|v| *v > 0 && *v <= u32::MAX as u64)
        .ok_or("Malformed provider refactor line")?;
    let offset = value
        .get("offset")
        .and_then(Value::as_u64)
        .filter(|v| *v > 0 && *v <= u32::MAX as u64)
        .ok_or("Malformed provider refactor offset")?;
    Ok(json!({"line":line-1,"character":offset-1}))
}

pub(crate) fn edits(response: &Value) -> Result<Value, String> {
    if response.get("success") != Some(&Value::Bool(true))
        || response.get("command").and_then(Value::as_str) != Some("getEditsForRefactor")
    {
        return Err("The provider did not return a successful refactoring reading".into());
    }
    let body = response
        .get("body")
        .and_then(Value::as_object)
        .ok_or("Missing refactoring reading")?;
    if let Some(reason) = body.get("notApplicableReason").and_then(Value::as_str) {
        return Err(format!("Refactoring unavailable: {reason}"));
    }
    let files = body
        .get("edits")
        .and_then(Value::as_array)
        .filter(|v| !v.is_empty() && v.len() <= 1000)
        .ok_or("Choose a bounded nonempty refactoring")?;
    let mut seen = BTreeSet::new();
    let mut changes = Vec::new();
    let mut total = 0usize;
    let mut count = 0usize;
    for file in files {
        let path = file
            .get("fileName")
            .and_then(Value::as_str)
            .filter(|s| s.len() <= 16384)
            .ok_or("Malformed refactor filename")?;
        let uri =
            Url::from_file_path(path).map_err(|_| "Refactor filename is not an absolute file")?;
        if !seen.insert(uri.to_string()) {
            return Err(
                "Repeated refactor file would require unobserved intermediate source".into(),
            );
        }
        let declared_new = match file.get("isNewFile") {
            None | Some(Value::Bool(false)) => false,
            Some(Value::Bool(true)) => true,
            _ => return Err("Malformed refactor file disposition".into()),
        };
        let entries = file
            .get("textChanges")
            .and_then(Value::as_array)
            .ok_or("Malformed refactor text changes")?;
        let zero = |value: &Value| value.get("line").and_then(Value::as_u64) == Some(0)
            && value.get("offset").and_then(Value::as_u64) == Some(0);
        let sentinel = entries.len() == 1 && zero(&entries[0]["start"]) && zero(&entries[0]["end"]);
        if sentinel && file.get("isNewFile") == Some(&Value::Bool(false)) {
            return Err("Contradictory new-file refactor sentinel".into());
        }
        let new = declared_new || sentinel;
        let mut mapped: Vec<Value> = Vec::new();
        for edit in entries {
            count += 1;
            let text = edit
                .get("newText")
                .and_then(Value::as_str)
                .ok_or("Malformed refactor replacement")?;
            total = total.saturating_add(text.len());
            if count > 10000 || total > 32 * 1024 * 1024 {
                return Err("The complete refactoring exceeds the review budget".into());
            }
            let start = if sentinel { json!({"line":0,"character":0}) } else { position(&edit["start"])? };
            let end = if sentinel { json!({"line":0,"character":0}) } else { position(&edit["end"])? };
            let pair = |p: &Value| {
                (
                    p["line"].as_u64().unwrap(),
                    p["character"].as_u64().unwrap(),
                )
            };
            if pair(&start) > pair(&end) || new && (pair(&start) != (0, 0) || pair(&end) != (0, 0))
            {
                return Err("Invalid refactor source range".into());
            }
            if let Some(previous) = mapped.last_mut() {
                let before = pair(&previous["range"]["end"]);
                if pair(&start) < before {
                    return Err(
                        "Overlapping or out-of-order TypeScript changes cannot be previewed".into(),
                    );
                }
                if pair(&start) == before {
                    let joined = format!("{}{}", previous["newText"].as_str().unwrap(), text);
                    previous["range"]["end"] = end;
                    previous["newText"] = json!(joined);
                    continue;
                }
            }
            mapped.push(json!({"range":{"start":start,"end":end},"newText":text}));
        }
        if new {
            changes.push(json!({"kind":"create","uri":uri}));
        }
        changes.push(json!({"textDocument":{"uri":uri,"version":null},"edits":mapped}));
    }
    Ok(json!({"documentChanges":changes}))
}
</ui:copy>

TypeScript applies its ordered text changes in reverse. Concatenate only exactly
adjacent ranges, retaining the provider order of coincident insertions, before
shared projection. This represents getter/setter generation as one unambiguous
edit without weakening the shared overlap rule or borrowing an intermediate file.
Overlapping or out-of-order members still refuse the whole reading.

## Refuse unsupported authority and partial answers

These checks establish the adapter's request and decoding policy. Genuine provider
and host preview/application evidence is required separately before claiming the
usable refactoring path.

<ui:copy id="provider-refactor-read-only-witness">
#[cfg(test)]
mod tests {
    use super::*;
    fn action() -> Value {
        json!({"title":"Extract","command":{"command":"_typescript.applyRefactoring","arguments":[{"file":"/workspace/a.ts","refactor":"Extract Symbol","action":"function_scope_0"}]}})
    }
    fn response() -> Value {
        json!({"success":true,"command":"getEditsForRefactor","body":{"edits":[{"fileName":"/workspace/a.ts","textChanges":[{"start":{"line":1,"offset":2},"end":{"line":1,"offset":4},"newText":"helper()"}]},{"fileName":"/workspace/helper.ts","isNewFile":true,"textChanges":[{"start":{"line":1,"offset":1},"end":{"line":1,"offset":1},"newText":"export function helper() {}\n"}]}]}})
    }
    #[test]
    fn held_bound_read_never_dispatches_application() {
        let uri = Url::parse("file:///workspace/a.ts").unwrap();
        let value = query(&action(), &uri, true).unwrap();
        assert_eq!(value["command"], "typescript.tsserverRequest");
        assert_eq!(value["arguments"][0], "getEditsForRefactor");
        assert_eq!(value["arguments"][1], action()["command"]["arguments"][0]);
        assert!(query(&action(), &uri, false).is_none());
        assert!(
            query(
                &action(),
                &Url::parse("file:///workspace/other.ts").unwrap(),
                true
            )
            .is_none()
        );
        let mut mixed = action();
        mixed["edit"] = json!({"changes":{}});
        assert!(query(&mixed, &uri, true).is_none());
    }
    #[test]
    fn complete_edits_preserve_creation_and_utf16_coordinates() {
        let value = edits(&response()).unwrap();
        assert_eq!(value["documentChanges"].as_array().unwrap().len(), 3);
        assert_eq!(
            value["documentChanges"][0]["edits"][0]["range"]["start"],
            json!({"line":0,"character":1})
        );
        assert_eq!(value["documentChanges"][1]["kind"], "create");
    }
    #[test]
    fn ordered_boundary_insertions_become_one_guarded_edit() {
        let mut value = response();
        value["body"]["edits"] = json!([{"fileName":"/workspace/a.ts","textChanges":[
            {"start":{"line":1,"offset":2},"end":{"line":1,"offset":4},"newText":"field"},
            {"start":{"line":1,"offset":4},"end":{"line":1,"offset":4},"newText":"getter"},
            {"start":{"line":1,"offset":4},"end":{"line":1,"offset":4},"newText":"setter"}
        ]}]);
        let edit = edits(&value).unwrap();
        assert_eq!(
            edit["documentChanges"][0]["edits"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            edit["documentChanges"][0]["edits"][0]["newText"],
            "fieldgettersetter"
        );
        value["body"]["edits"][0]["textChanges"][1]["start"]["offset"] = json!(3);
        assert!(edits(&value).is_err());
    }
    #[test]
    fn native_zero_position_new_file_is_an_ordered_creation_only() {
        let mut reading=response();
        let member=&mut reading["body"]["edits"][1];
        member.as_object_mut().unwrap().remove("isNewFile");
        member["textChanges"][0]["start"]=json!({"line":0,"offset":0});
        member["textChanges"][0]["end"]=json!({"line":0,"offset":0});
        let decoded=edits(&reading).unwrap();
        assert_eq!(decoded["documentChanges"][1],json!({"kind":"create","uri":"file:///workspace/helper.ts"}));
        assert_eq!(decoded["documentChanges"][2]["edits"][0]["range"],json!({"start":{"line":0,"character":0},"end":{"line":0,"character":0}}));
        reading["body"]["edits"][1]["isNewFile"]=json!(false);
        assert!(edits(&reading).is_err());
    }
    #[test]
    fn incomplete_or_multiple_zero_position_members_refuse_whole_reading() {
        for key in ["line","offset"] {
            let mut reading=response();reading["body"]["edits"][1]["textChanges"][0]["start"][key]=json!(0);
            assert!(edits(&reading).is_err());
        }
        let mut reading=response();let member=&mut reading["body"]["edits"][1];
        member["textChanges"]=json!([
          {"start":{"line":0,"offset":0},"end":{"line":0,"offset":0},"newText":"first"},
          {"start":{"line":0,"offset":0},"end":{"line":0,"offset":0},"newText":"second"}
        ]);
        assert!(edits(&reading).is_err());
    }
    #[test]
    fn malformed_later_member_refuses_the_complete_reading() {
        let mut value = response();
        value["body"]["edits"][1]["textChanges"][0]["start"]["offset"] = json!(false);
        assert!(edits(&value).is_err());
        let mut value = response();
        value["body"]["edits"][1]["fileName"] = json!("/workspace/a.ts");
        assert!(edits(&value).is_err());
        let mut value = response();
        value["success"] = json!(false);
        assert!(edits(&value).is_err());
    }
}
</ui:copy>
<ui:file path="crates/hick-lsp/src/provider_refactors.rs"><ui:paste select="#provider-refactor-read-only-model"/><ui:paste select="#provider-refactor-read-only-witness"/></ui:file>
</ui:doc>
