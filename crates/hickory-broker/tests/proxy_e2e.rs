//! The broker over a real socket: allow forwards, deny refuses with a
//! sentence, and nothing else is a road.
//!
//! Protects `docs/guarantees/collaboration/one-road-out-with-a-toll-booth.md`.

use std::sync::Arc;

use hickory_broker::policy::Verb;
use hickory_broker::{Broker, BrokerLog, Policy};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};

/// An upstream that echoes whatever it is sent — the thing a CONNECT tunnel
/// is supposed to reach.
async fn echo_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                while let Ok(n) = sock.read(&mut buf).await {
                    if n == 0 || sock.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
            });
        }
    });
    port
}

async fn start(policy: Policy, dir: &std::path::Path) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let broker = Arc::new(Broker {
        policy,
        log: Arc::new(BrokerLog::at(&dir.join("broker.jsonl"))),
        now: Arc::new(|| "2026-08-23T00:00:00Z".to_string()),
    });
    tokio::spawn(async move {
        let _ = broker.serve(listener).await;
    });
    port
}

async fn connect_through(port: u16, target: &str) -> (String, TcpStream) {
    let mut sock = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    sock.write_all(format!("CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n").as_bytes())
        .await
        .unwrap();
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        let n = sock.read(&mut byte).await.unwrap();
        if n == 0 {
            break;
        }
        head.push(byte[0]);
    }
    (String::from_utf8_lossy(&head).to_string(), sock)
}

#[tokio::test]
async fn an_allowed_host_is_forwarded_byte_for_byte() {
    let dir = tempfile::tempdir().unwrap();
    let upstream = echo_server().await;
    let mut policy = Policy::default();
    policy.set("127.0.0.1", Verb::Allow, false).unwrap();
    let port = start(policy, dir.path()).await;

    let (head, mut tunnel) = connect_through(port, &format!("127.0.0.1:{upstream}")).await;
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");

    tunnel.write_all(b"hello").await.unwrap();
    let mut buf = [0u8; 5];
    tunnel.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"hello");

    // The broker is OUTSIDE the connection: it logs a hostname and a byte
    // count, never a body.
    drop(tunnel);
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    let entries = BrokerLog::at(&dir.path().join("broker.jsonl")).read();
    let allowed = entries.iter().find(|e| e.verb == Verb::Allow).unwrap();
    assert_eq!(allowed.host, "127.0.0.1");
    assert!(allowed.bytes.unwrap_or(0) >= 5);
    let raw = std::fs::read_to_string(dir.path().join("broker.jsonl")).unwrap();
    assert!(!raw.contains("hello"), "the log holds a body: {raw}");
}

#[tokio::test]
async fn an_unlisted_host_is_denied_with_a_sentence_the_agent_can_repeat() {
    // An agent that receives a timeout invents a reason; an agent that
    // receives a sentence repeats it.
    let dir = tempfile::tempdir().unwrap();
    let port = start(Policy::default(), dir.path()).await;
    let (head, mut sock) = connect_through(port, "raw.githubusercontent.com:443").await;
    assert!(head.starts_with("HTTP/1.1 403"), "{head}");

    let mut body = String::new();
    sock.read_to_string(&mut body).await.unwrap();
    assert!(body.contains("raw.githubusercontent.com"), "{body}");
    assert!(
        body.contains("ask the engineer to allow that host"),
        "{body}"
    );
    assert!(body.contains("one road out"), "{body}");
    // Never "airgapped": a machine that talks to a model is on a network.
    assert!(!body.to_lowercase().contains("airgap"), "{body}");
}

#[tokio::test]
async fn ask_and_substitute_are_denied_rather_than_left_hanging() {
    // `ask` needs the fleet channel and `substitute` needs a CA; neither is
    // built. An agent that hangs for an hour produces a half-finished turn
    // nobody can explain, so both refuse and say why.
    let dir = tempfile::tempdir().unwrap();
    let mut policy = Policy::default();
    policy.set("ask.example", Verb::Ask, false).unwrap();
    policy.set("sub.example", Verb::Substitute, true).unwrap();
    let port = start(policy, dir.path()).await;

    for (host, expected) in [
        ("ask.example", "ASK a human"),
        ("sub.example", "SUBSTITUTE"),
    ] {
        let (head, mut sock) = connect_through(port, &format!("{host}:443")).await;
        assert!(head.starts_with("HTTP/1.1 403"), "{host}: {head}");
        let mut body = String::new();
        sock.read_to_string(&mut body).await.unwrap();
        assert!(body.contains(expected), "{host}: {body}");
    }
    let entries = BrokerLog::at(&dir.path().join("broker.jsonl")).read();
    let sub = entries.iter().find(|e| e.verb == Verb::Substitute).unwrap();
    assert!(
        sub.reason
            .as_deref()
            .unwrap()
            .contains("no credential was used")
    );
}

#[tokio::test]
async fn anything_other_than_connect_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let port = start(Policy::default(), dir.path()).await;
    let mut sock = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    sock.write_all(b"GET http://example.com/ HTTP/1.1\r\nHost: x\r\n\r\n")
        .await
        .unwrap();
    let mut answer = String::new();
    sock.read_to_string(&mut answer).await.unwrap();
    assert!(answer.starts_with("HTTP/1.1 405"), "{answer}");
    assert!(answer.contains("CONNECT only"), "{answer}");
}

#[tokio::test]
async fn an_allowed_host_that_cannot_be_reached_says_it_is_the_network() {
    let dir = tempfile::tempdir().unwrap();
    let mut policy = Policy::default();
    policy.set("127.0.0.1", Verb::Allow, false).unwrap();
    let port = start(policy, dir.path()).await;
    // Port 1 on loopback: allowed by policy, nothing listening.
    let (head, mut sock) = connect_through(port, "127.0.0.1:1").await;
    assert!(head.starts_with("HTTP/1.1 502"), "{head}");
    let mut body = String::new();
    sock.read_to_string(&mut body).await.unwrap();
    assert!(
        body.contains("That is the network, not the policy"),
        "{body}"
    );
}
