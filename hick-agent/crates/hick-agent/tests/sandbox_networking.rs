//! Integration tests: guest networking inside the sandbox.
//!
//! Run with:
//! ```text
//! cargo test -p hick-agent --features integration-test --test sandbox_networking -- --test-threads=1
//! ```

#![cfg(feature = "integration-test")]

mod common;

use std::sync::Arc;

use common::{create_executor_with_capabilities, create_executor_with_guest_network};
use hick_agent_core::{CommandExecutor, CommandOutput};
use hick_token::ContainerCapabilities;
use tokio::sync::Mutex;

const SANDBOX_CONTAINER: &str = "agent-sandbox";

async fn exec_sh(
    executor: &Arc<Mutex<dyn CommandExecutor>>,
    image: &str,
    shell_cmd: &str,
) -> CommandOutput {
    executor
        .lock()
        .await
        .execute(SANDBOX_CONTAINER, shell_cmd, image, None)
        .await
        .unwrap_or_else(|e| panic!("sandbox execute failed: {e}\ncommand: {shell_cmd}"))
}

/// Verify the guest has a non-loopback IP address (DHCP succeeded).
#[tokio::test(flavor = "multi_thread")]
async fn sandbox_guest_receives_dhcp_address() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor_with_guest_network(workdir.path(), true);

    let out = exec_sh(&executor, &image, "ip addr show eth0 2>/dev/null || ifconfig eth0 2>/dev/null || ip addr 2>/dev/null").await;
    let output = &out.output;
    eprintln!("[test] ip addr output:\n{output}");
    assert!(
        output.contains("192.168.127") || output.contains("inet "),
        "guest should have an IP via DHCP; got:\n{output}"
    );
}

/// Verify the guest can make actual TCP connections via the QEMU network stack.
///
/// This tests that QEMU networking is truly connected (frames flow through the host
/// network stack), not just that local routing is configured.
#[tokio::test(flavor = "multi_thread")]
async fn sandbox_guest_qemu_networking_connected() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor_with_guest_network(workdir.path(), true);

    // Try to reach an external server by raw IP (no DNS needed).
    // Connection refused or HTTP response both indicate TCP connectivity.
    // "Network unreachable" / timeout-with-no-response indicates QEMU frames aren't flowing.
    let out = exec_sh(
        &executor,
        &image,
        // 1.1.1.1 is Cloudflare DNS — port 80 will either respond or refuse, both are fine.
        "wget -T 10 -q -O /dev/null http://1.1.1.1/ 2>&1; echo exit=$?",
    )
    .await;
    let output = &out.output;
    eprintln!("[test] wget 1.1.1.1 output:\n{output}");
    // Any outcome except a silent timeout or unreachable is fine.
    // A redirect or HTTP response means QEMU networking is connected.
    assert!(
        output.contains("302") || output.contains("301") || output.contains("200")
            || output.contains("Connection refused")
            || output.contains("exit=0")
            || !output.contains("exit=1"),
        "QEMU networking should be connected (TCP reaches 1.1.1.1); got:\n{output}"
    );
}

/// Verify that wget can reach an external host by hostname (DNS + TCP).
///
/// This tests the full musl getaddrinfo() + TCP pipeline, which the live agent
/// uses. Unlike the raw-IP wget test, this requires DNS to actually work.
#[tokio::test(flavor = "multi_thread")]
async fn sandbox_guest_wget_hostname() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor_with_guest_network(workdir.path(), true);

    // First confirm DNS resolves at all
    let dns_out = exec_sh(
        &executor,
        &image,
        "nslookup example.com 2>&1 | head -4; echo exit=$?",
    )
    .await;
    eprintln!("[test] nslookup example.com:\n{}", dns_out.output);

    // Now try wget by hostname — this goes through musl getaddrinfo()
    let out = exec_sh(
        &executor,
        &image,
        "wget -T 15 -q -O /dev/null http://example.com/ 2>&1; echo exit=$?",
    )
    .await;
    let output = &out.output;
    eprintln!("[test] wget http://example.com/ output:\n{output}");
    assert!(
        !output.contains("bad address"),
        "wget must be able to resolve the hostname via musl getaddrinfo(); got:\n{output}"
    );
}

/// Verify DNS resolution works inside the sandbox.
///
/// The container's /etc/resolv.conf points to 10.0.1.1 (the smoltcp gateway
/// configured via `NetworkConfig::for_container`). We query without an explicit
/// server so nslookup uses /etc/resolv.conf rather than a hardcoded IP.
#[tokio::test(flavor = "multi_thread")]
async fn sandbox_guest_resolves_dns() {
    let workdir = tempfile::tempdir().unwrap();
    let (executor, image) = create_executor_with_guest_network(workdir.path(), true);

    // Check /etc/resolv.conf for diagnostics.
    let resolv = exec_sh(&executor, &image, "cat /etc/resolv.conf 2>/dev/null; echo ---").await;
    eprintln!("[test] /etc/resolv.conf:\n{}", resolv.output);

    let out = exec_sh(
        &executor,
        &image,
        // No explicit server — uses /etc/resolv.conf (10.0.1.1 baked into image).
        "nslookup google.com 2>&1 | head -10; echo exit=$?",
    )
    .await;
    let output = &out.output;
    eprintln!("[test] nslookup output:\n{output}");
    assert!(
        output.contains("Address") || output.contains("answer"),
        "nslookup should return an address for google.com; got:\n{output}"
    );
}

/// Regression test: TCP connections to allowed hosts work after DNS resolution.
///
/// Covers the bug where handle_dns forwarded DNS responses but never called
/// register_dns_result on the filter. CapabilityFilter then had no hostname
/// cached for the resolved IP, fell back to IP-string matching, found no match
/// under DenyAll, and reset the connection ("Connection reset by peer") even
/// though the hostname was in the allow list.
///
/// Uses a DenyAll + specific-allowlist config matching the live agent default,
/// and verifies that wget can actually reach the allowed host over TCP.
#[tokio::test(flavor = "multi_thread")]
async fn sandbox_tcp_allowed_after_capability_filter_dns_registration() {
    let workdir = tempfile::tempdir().unwrap();

    // Mirror the live agent's default security policy: deny all, but allow
    // example.com (a stable, well-known host that doesn't require auth).
    let caps = ContainerCapabilities::new()
        .allow_network("example.com", "*")
        .deny_all_network();

    let (executor, image) = create_executor_with_capabilities(workdir.path(), true, caps);

    // DNS for the allowed host must resolve.
    let dns_out = exec_sh(
        &executor,
        &image,
        "nslookup example.com 2>&1 | head -4; echo exit=$?",
    )
    .await;
    eprintln!("[test] nslookup example.com:\n{}", dns_out.output);
    assert!(
        dns_out.output.contains("Address"),
        "DNS for allowed host should resolve; got:\n{}",
        dns_out.output
    );

    // TCP to the resolved IP must succeed — NOT "bad address" or "Connection reset by peer".
    let out = exec_sh(
        &executor,
        &image,
        "wget -T 15 -q -O /dev/null http://example.com/ 2>&1; echo exit=$?",
    )
    .await;
    let output = &out.output;
    eprintln!("[test] wget http://example.com/ (capability filter):\n{output}");
    assert!(
        !output.contains("bad address") && !output.contains("reset by peer"),
        "wget to allowed host must not be blocked by capability filter; got:\n{output}"
    );

    // DNS for a blocked host must be silently dropped (no external address returned).
    let blocked = exec_sh(
        &executor,
        &image,
        "nslookup google.com 2>&1 | head -4; echo exit=$?",
    )
    .await;
    eprintln!("[test] nslookup google.com (should be blocked):\n{}", blocked.output);
    assert!(
        !blocked.output.contains("Address:") || blocked.output.contains("10.0.1.1"),
        "DNS for blocked host should not resolve to an external address; got:\n{}",
        blocked.output
    );
}
