//! Quick test: wasmtime CLI subprocess, WITH and WITHOUT --net=socket.
//! Uses blocking reads on a separate thread.

use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

fn test_cli(label: &str, extra_wasm_args: &[&str]) {
    eprintln!("\n=== {label} ===");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wasm_path = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("c2w/converted_images/alpine/3.23.2.wasm");

    let mut cmd = Command::new("wasmtime");
    cmd.arg("run")
        .arg("-S")
        .arg("preview2=n")
        .arg("-S")
        .arg("tcplisten=127.0.0.1:1234")
        .arg("--env=LISTEN_FDS=1")
        .arg("--")
        .arg(&wasm_path);

    for arg in extra_wasm_args {
        cmd.arg(arg);
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    eprintln!("[{label}] Starting wasmtime...");
    let mut child = cmd.spawn().expect("failed to start wasmtime");
    eprintln!("[{label}] PID {}", child.id());

    eprintln!("[{label}] Waiting 30s for module compilation...");
    std::thread::sleep(Duration::from_secs(30));

    eprintln!("[{label}] Connecting...");
    let stream =
        TcpStream::connect_timeout(&"127.0.0.1:1234".parse().unwrap(), Duration::from_secs(5));
    let stream = match stream {
        Ok(s) => {
            eprintln!("[{label}] Connected!");
            s
        }
        Err(e) => {
            eprintln!("[{label}] FAILED to connect: {e}");
            let _ = child.kill();
            let _ = child.wait();
            return;
        }
    };

    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut header = [0u8; 4];
    let mut frame_count = 0u64;
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(20) {
        match (&stream).read_exact(&mut header) {
            Ok(()) => {
                let len = u32::from_be_bytes(header) as usize;
                frame_count += 1;
                let mut payload = vec![0u8; len];
                let _ = (&stream).read_exact(&mut payload);
                if len >= 14 {
                    let et = u16::from_be_bytes([payload[12], payload[13]]);
                    eprintln!("[{label}] Frame #{frame_count}: {len}B, et=0x{et:04x}");
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                continue;
            }
            Err(e) => {
                eprintln!("[{label}] Read error: {e}");
                break;
            }
        }
    }

    eprintln!(
        "[{label}] RESULT: {frame_count} frames in {:?}",
        start.elapsed()
    );

    let _ = child.kill();
    let _ = child.wait();
}

fn main() {
    test_cli("WITH --net=socket", &["--net=socket"]);
    std::thread::sleep(Duration::from_secs(2)); // Let port free up
    test_cli("WITHOUT --net=socket", &[]);
}
