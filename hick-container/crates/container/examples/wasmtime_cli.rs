//! Test using wasmtime CLI directly to verify the socket approach works
//!
//! This spawns wasmtime as a subprocess with the same flags c2w-net uses.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    eprintln!("[test] Starting wasmtime CLI test...");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wasm_path = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("c2w/converted_images/alpine/3.23.2.wasm");

    eprintln!("[test] WASM path: {:?}", wasm_path);

    // Start wasmtime with tcplisten like c2w-net does
    // The container2wasm docs say --net=socket is required for networking
    let mut child = Command::new("wasmtime")
        .arg("run")
        .arg("-S")
        .arg("preview2=n")
        .arg("-S")
        .arg("tcplisten=127.0.0.1:1234")
        .arg("--env=LISTEN_FDS=1")
        .arg("--")
        .arg(&wasm_path)
        .arg("--net=socket")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    eprintln!("[test] wasmtime started with PID {}", child.id());

    // Give it a moment to start
    thread::sleep(Duration::from_secs(2));

    // Try to connect with retries
    eprintln!("[test] Attempting to connect to 127.0.0.1:1234...");
    let stream = loop {
        match std::net::TcpStream::connect_timeout(
            &"127.0.0.1:1234".parse()?,
            Duration::from_secs(2),
        ) {
            Ok(s) => {
                eprintln!("[test] Connected!");
                break s;
            }
            Err(e) => {
                eprintln!("[test] Connection attempt failed: {e}, retrying...");
                thread::sleep(Duration::from_secs(1));
            }
        }
    };

    // Read with timeout - use blocking reads with read timeout
    eprintln!("[test] Reading with blocking reads...");
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut buf = [0u8; 4096];
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        match std::io::Read::read(&mut &stream, &mut buf) {
            Ok(0) => {
                eprintln!("[test] Connection closed (0 bytes)");
                break;
            }
            Ok(n) => {
                eprintln!("[test] Read {} bytes: {:02x?}", n, &buf[..n.min(64)]);
                // Parse QEMU protocol: 4-byte big-endian length prefix
                if n >= 4 {
                    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
                    eprintln!("[test] QEMU frame length: {}", len);
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                eprintln!("[test] Read timeout, trying again...");
            }
            Err(e) => {
                eprintln!("[test] Read error: {e}");
                break;
            }
        }
    }
    eprintln!("[test] Done reading, elapsed: {:?}", start.elapsed());

    // Read stdout/stderr
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().take(10) {
            if let Ok(line) = line {
                eprintln!("[wasmtime stdout] {}", line);
            }
        }
    }

    eprintln!("[test] Killing wasmtime...");
    let _ = child.kill();
    let _ = child.wait();
    eprintln!("[test] Done");

    Ok(())
}
