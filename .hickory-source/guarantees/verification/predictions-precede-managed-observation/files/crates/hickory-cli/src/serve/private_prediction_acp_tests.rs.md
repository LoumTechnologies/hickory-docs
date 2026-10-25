<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/private_prediction_acp_tests.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `5f9bb74e8b9f006c8142be8d97a934f451d9a23c108d567211a9112e5214a767`.

<book:entry path="crates/hickory-cli/src/serve/private_prediction_acp_tests.rs">
//! Guarantee: verification/predictions-precede-managed-observation.md
use super::*;

#[test]
fn private_mcp_permission_is_one_shot_and_never_general_consent() {
    // Guarantee: verification/predictions-precede-managed-observation.md.
    // An adapter envelope cannot approve an edit or bypass Bridge::call review.
    let mut policy = McpPermissions {
        session: Some("private-session".into()),
        allowed: ["read_doc".into(), "edit_doc".into()].into(),
        ..Default::default()
    };
    let notification = |server: &str, tool: &str, session: &str| json!({"method":"session/update","params":{"sessionId":session,"update":{"sessionUpdate":"tool_call","toolCallId":"call-1","status":"in_progress","rawInput":{"server":server,"tool":tool,"arguments":{}}}}});
    let request = json!({"sessionId":"private-session","toolCall":{"toolCallId":"call-1","kind":"execute"},"_meta":{"is_mcp_tool_approval":true},"options":[{"kind":"allow_always","optionId":"forever"},{"kind":"allow_once","optionId":"one"}]});
    assert_eq!(policy.outcome(&request)["outcome"], "cancelled");
    for (server, tool, session) in [
        ("ordinary-server", "read_doc", "private-session"),
        (
            "hickory-private-prediction",
            "terminal/create",
            "private-session",
        ),
        ("hickory-private-prediction", "read_doc", "old-session"),
    ] {
        policy.observe(&notification(server, tool, session));
        assert_eq!(policy.outcome(&request)["outcome"], "cancelled");
    }
    for tool in ["read_doc", "edit_doc"] {
        policy.observe(&notification(
            "hickory-private-prediction",
            tool,
            "private-session",
        ));
        assert_eq!(
            policy.outcome(&request),
            json!({"outcome":"selected","optionId":"one"})
        );
        assert_eq!(policy.outcome(&request)["outcome"], "cancelled");
    }
    policy.observe(&notification(
        "hickory-private-prediction",
        "read_doc",
        "private-session",
    ));
    policy.observe(&notification(
        "ordinary-server",
        "read_doc",
        "private-session",
    ));
    assert_eq!(policy.outcome(&request)["outcome"], "cancelled");
    for altered in [
        json!({"toolCall":{"toolCallId":"call-1","kind":"fetch"}}),
        json!({"sessionId":"other"}),
        json!({"_meta":{"is_mcp_tool_approval":false}}),
        json!({"options":[{"kind":"allow_always","optionId":"forever"}]}),
    ] {
        policy.observe(&notification(
            "hickory-private-prediction",
            "read_doc",
            "private-session",
        ));
        let mut invalid = request.clone();
        for (key, value) in altered.as_object().unwrap() {
            invalid[key] = value.clone();
        }
        assert_eq!(policy.outcome(&invalid)["outcome"], "cancelled");
    }
    policy.observe(&notification(
        "hickory-private-prediction",
        "read_doc",
        "private-session",
    ));
    policy.observe(&json!({"method":"session/update","params":{"sessionId":"private-session","update":{"sessionUpdate":"tool_call_update","toolCallId":"call-1","status":"completed"}}}));
    assert_eq!(policy.outcome(&request)["outcome"], "cancelled");
}
pub(crate) async fn managed_boundary(state: LocalState) {
    let bridge = Bridge {
        state: state.clone(),
        window: None,
        actor: "acp-private:test".into(),
        operation: Arc::default(),
        cancel: Arc::new(AtomicBool::new(false)),
        calls: Arc::default(),
    };
    let host = bridge.clone().start().await.unwrap();
    let client = reqwest::Client::new();
    let response = client
        .post(&host.url)
        .json(&json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    let tools = response["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 4);
    assert!(tools.iter().all(|tool| {
        ["read_doc", "edit_doc", "verify", "seal_prediction"]
            .contains(&tool["name"].as_str().unwrap())
    }));
    let bound = state.private_prediction.clone().unwrap();
    for name in [
        "run_doc",
        "evaluate",
        "reveal",
        "read_file",
        "terminal/create",
    ] {
        assert_eq!(
            bridge.call(&json!({"name":name})).await.unwrap()["isError"],
            true
        );
    }
    let read = bridge.call(&json!({"name":"read_doc"})).await.unwrap();
    assert_ne!(read["isError"], true);
    let source = read["structuredContent"]["source"]
        .as_str()
        .unwrap()
        .to_owned();
    let snapshot = read["structuredContent"]["snapshot"].clone();
    let task_bridge = bridge.clone();
    let proposed = source.replace("printf 100", "printf 99");
    let task = tokio::spawn(async move {
        task_bridge.call(&json!({"name":"edit_doc","arguments":{"whole":"true","snapshot":snapshot,"input":proposed}})).await
    });
    let id = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let view = bound.edits.snapshot();
            if let Some(id) = view["changes"]
                .as_array()
                .and_then(|changes| changes.last())
                .and_then(|change| change["id"].as_str())
            {
                break id.to_owned();
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(bound.source_path()).unwrap(),
        source
    );
    // The human's replacement bytes, rather than the model's proposal, are applied.
    let replacement = source.replace("printf 100", "printf 98");
    bound.edits.revise(&id, replacement.clone()).unwrap();
    bound.edits.resolve(&id, Ok(())).unwrap();
    assert_ne!(task.await.unwrap().unwrap()["isError"], true);
    assert_eq!(
        std::fs::read_to_string(bound.source_path()).unwrap(),
        replacement
    );
    // A refusal does not mutate source, even though the agent proposed it.
    let read = bridge.call(&json!({"name":"read_doc"})).await.unwrap();
    let snapshot = read["structuredContent"]["snapshot"].clone();
    let next_bridge = bridge.clone();
    let task = tokio::spawn(async move {
        next_bridge.call(&json!({"name":"edit_doc","arguments":{"whole":"true","snapshot":snapshot,"input":"DECLINED SOURCE"}})).await
    });
    let declined = wait_review(&bound.edits).await;
    bound
        .edits
        .resolve(&declined, Err("Human declined".into()))
        .unwrap();
    assert!(task.await.unwrap().is_err());
    assert_eq!(
        std::fs::read_to_string(bound.source_path()).unwrap(),
        replacement
    );

    let read = bridge.call(&json!({"name":"read_doc"})).await.unwrap();
    let snapshot = read["structuredContent"]["snapshot"].clone();
    let next_bridge = bridge.clone();
    let task = tokio::spawn(async move {
        next_bridge.call(&json!({"name":"edit_doc","arguments":{"whole":"true","snapshot":snapshot,"input":"STALE APPROVAL"}})).await
    });
    let stale = wait_review(&bound.edits).await;
    let host_replacement = replacement.replace("printf 98", "printf 97");
    let host_result = bound.mcp_tool(state.executor.clone(), &json!({"name":"edit_doc","arguments":{"whole":"true","snapshot":read["structuredContent"]["snapshot"],"input":host_replacement}})).await;
    assert_ne!(host_result["isError"], true);
    bound.edits.resolve(&stale, Ok(())).unwrap();
    assert!(task.await.unwrap().is_err());
    assert_eq!(
        std::fs::read_to_string(bound.source_path()).unwrap(),
        host_replacement
    );
    bridge.cancel.store(true, Ordering::SeqCst);
    assert!(bridge.call(&json!({"name":"read_doc"})).await.is_err());
}

pub(crate) async fn protocol_boundary(state: LocalState) {
    let directory = tempfile::tempdir().unwrap();
    let audit = tempfile::tempdir().unwrap();
    let audit_path = audit.path().join("protocol.json");
    let script = r#"
import json,sys,urllib.request,os
url=None
checks={}
def send(v): print(json.dumps(v),flush=True)
def reply(m,v): send({'jsonrpc':'2.0','id':m['id'],'result':v})
for line in sys.stdin:
 m=json.loads(line)
 if m['method']=='initialize':
  caps=m['params']['clientCapabilities']
  checks['filesystem_refused']=caps['fs']=={'readTextFile':False,'writeTextFile':False}
  checks['terminal_refused']=caps['terminal']==False
  reply(m,{'protocolVersion':1,'authMethods':[{'id':'fixture-api-key'}],'agentCapabilities':{'mcpCapabilities':{'http':True}}})
 elif m['method']=='authenticate':
  checks['explicit_authentication']=m['params']['methodId']=='fixture-api-key';reply(m,{})
 elif m['method']=='session/new':
  checks['fresh_context']=os.listdir(m['params']['cwd'])==[] and os.environ['HOME']==m['params']['cwd']
  url=m['params']['mcpServers'][0]['url'];reply(m,{'sessionId':'fresh-private'})
 elif m['method']=='session/prompt':
  send({'jsonrpc':'2.0','id':'fs-test','method':'fs/read_text_file','params':{'path':'/private/ledger'}})
  checks['filesystem_call_refused']=json.loads(sys.stdin.readline())['error']['code']==-32601
  send({'jsonrpc':'2.0','id':'permission-test','method':'session/request_permission','params':{}})
  checks['permission_cancelled']=json.loads(sys.stdin.readline())['result']['outcome']['outcome']=='cancelled'
  send({'jsonrpc':'2.0','method':'session/update','params':{'sessionId':'fresh-private','update':{'sessionUpdate':'tool_call','toolCallId':'private-read','status':'in_progress','rawInput':{'server':'hickory-private-prediction','tool':'read_doc','arguments':{}}}}})
  permission={'sessionId':'fresh-private','toolCall':{'toolCallId':'private-read','kind':'execute'},'_meta':{'is_mcp_tool_approval':True},'options':[{'kind':'allow_once','optionId':'only-this-call'}]}
  send({'jsonrpc':'2.0','id':'private-envelope','method':'session/request_permission','params':permission})
  checks['private_envelope_allowed_once']=json.loads(sys.stdin.readline())['result']['outcome']=={'outcome':'selected','optionId':'only-this-call'}
  send({'jsonrpc':'2.0','id':'replayed-envelope','method':'session/request_permission','params':permission})
  checks['envelope_replay_cancelled']=json.loads(sys.stdin.readline())['result']['outcome']['outcome']=='cancelled'
  request=urllib.request.Request(url,json.dumps({'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':'read_doc','arguments':{}}}).encode(),{'Content-Type':'application/json'})
  value=json.load(urllib.request.urlopen(request))
  checks['private_source_read']=value['result']['isError']==False and 'source_hash' in value['result']['structuredContent']
  send({'jsonrpc':'2.0','method':'session/update','params':{'update':{'sessionUpdate':'agent_message_chunk','content':{'type':'text','text':'UNVERIFIED MODEL CLAIM'}}}})
  with open(sys.argv[1],'w') as f: json.dump(checks,f)
  reply(m,{'stopReason':'end_turn'})
"#;
    let mut command = tokio::process::Command::new("python3");
    command
        .args(["-u", "-c", script])
        .arg(&audit_path)
        .current_dir(directory.path())
        .env("HOME", directory.path());
    let result = run(
        state,
        Prepared {
            command,
            directory,
            auth_method: Some("fixture-api-key".into()),
        },
        "Read only; never execute".into(),
        None,
        "acp-private:protocol".into(),
        Arc::new(AtomicBool::new(false)),
    )
    .await;
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("completion gate is non-passing")
    );
    let checks: Value = serde_json::from_slice(&std::fs::read(audit_path).unwrap()).unwrap();
    assert_eq!(checks.as_object().unwrap().len(), 9);
    assert!(
        checks
            .as_object()
            .unwrap()
            .values()
            .all(|value| value == true),
        "{checks}"
    );
}

#[test]
fn private_acp_process_profile_never_reuses_ordinary_arguments_or_home() {
    let mut agent = acp::config::AgentCommand {
        id: "qa".into(),
        name: "QA".into(),
        command: "python3".into(),
        args: vec!["--resume-ordinary".into()],
        workspace_filesystem: false,
        private_prediction_args: None,
        private_prediction_auth_method: None,
    };
    let config_dir = tempfile::tempdir().unwrap();
    assert!(
        prepare_command(agent.clone(), config_dir.path())
            .err()
            .unwrap()
            .to_string()
            .contains("explicit private_prediction_args")
    );
    agent.private_prediction_args = Some(vec!["-u".into()]);
    let prepared = prepare_command(agent.clone(), config_dir.path()).unwrap();
    assert_eq!(
        prepared.command.as_std().get_args().collect::<Vec<_>>(),
        vec![std::ffi::OsStr::new("-u")]
    );
    assert_eq!(
        prepared.command.as_std().get_current_dir(),
        Some(prepared.directory.path())
    );
    let env = prepared
        .command
        .as_std()
        .get_envs()
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        env.get(std::ffi::OsStr::new("HOME")).copied().flatten(),
        Some(prepared.directory.path().as_os_str())
    );
    assert!(
        !env.keys()
            .any(|key| key.to_string_lossy().starts_with("HICKORY_"))
    );
    agent.workspace_filesystem = true;
    assert!(prepare_command(agent, config_dir.path()).is_err());
}

async fn wait_review(edits: &acp::edits::Edits) -> String {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let view = edits.snapshot();
            if let Some(id) = view["changes"]
                .as_array()
                .and_then(|changes| changes.iter().find(|change| change["status"] == "pending"))
                .and_then(|change| change["id"].as_str())
            {
                break id.to_owned();
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap()
}
</book:entry>

</book:doc>
