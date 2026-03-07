use std::os::unix::net::UnixStream;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wasi_common::WasiCtx;
use wasmtime::{Engine, Module, Store};

fn make_engine() -> Engine {
    let mut config = wasmtime::Config::new();
    config.async_support(true);
    config.epoch_interruption(true);
    Engine::new(&config).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn test_wasi_common_fd_write() {
    let engine = make_engine();

    // Create stdout pipe
    let (host, wasi) = UnixStream::pair().unwrap();
    host.set_nonblocking(true).unwrap();
    let mut host = tokio::net::UnixStream::from_std(host).unwrap();

    let stdout = container::pipe::PipeWriter::from_unix_stream(wasi);

    let mut builder = wasi_common::tokio::WasiCtxBuilder::new();
    builder.stdout(Box::new(stdout));
    let wasi_ctx = builder.build();

    // A minimal WASM module that writes "hello" to stdout
    let wat = r#"
    (module
        (import "wasi_snapshot_preview1" "fd_write"
            (func $fd_write (param i32 i32 i32 i32) (result i32)))
        (memory (export "memory") 1)

        ;; iov at offset 0: { buf_ptr=8, buf_len=5 }
        (data (i32.const 0) "\08\00\00\00\05\00\00\00")
        ;; data at offset 8: "hello"
        (data (i32.const 8) "hello")

        (func (export "_start")
            ;; fd_write(fd=1, iovs=0, iovs_len=1, nwritten_ptr=16)
            (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 16))
            drop
        )
    )
    "#;

    let module = Module::new(&engine, wat).unwrap();

    let engine2 = engine.clone();
    let epoch = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
            engine2.increment_epoch();
        }
    });

    let engine3 = engine.clone();
    let task = tokio::spawn(async move {
        let mut linker = wasmtime::Linker::<WasiCtx>::new(&engine3);
        wasi_common::tokio::add_to_linker(&mut linker, |cx| cx).unwrap();

        let mut store = Store::new(&engine3, wasi_ctx);
        store.epoch_deadline_async_yield_and_update(1);

        linker.module_async(&mut store, "", &module).await.unwrap();
        let func = linker.get_default(&mut store, "").unwrap();
        let typed = func.typed::<(), ()>(&store).unwrap();
        typed.call_async(&mut store, ()).await.unwrap();
    });

    // Read from host side
    let mut buf = vec![0u8; 1024];
    let result = tokio::time::timeout(Duration::from_secs(5), host.read(&mut buf)).await;

    epoch.abort();

    match result {
        Ok(Ok(n)) => {
            let s = String::from_utf8_lossy(&buf[..n]);
            assert_eq!(&buf[..n], b"hello", "Expected 'hello', got {:?}", s);
        }
        Ok(Err(e)) => panic!("Read error: {e}"),
        Err(_) => panic!("Timeout waiting for output"),
    }

    task.await.unwrap();
}

/// Test that fd_read (stdin) + fd_write (stdout) work together with our pipe implementation.
/// This WAT module reads 5 bytes from stdin, then writes them to stdout.
#[tokio::test(flavor = "multi_thread")]
async fn test_wasi_common_fd_read_then_write() {
    let engine = make_engine();

    // Create stdin pipe (host writes, wasi reads)
    let (stdin_host, stdin_wasi) = container::pipe::stdin_pipe().unwrap();
    // Create stdout pipe (wasi writes, host reads)
    let (stdout_host_std, stdout_wasi) = UnixStream::pair().unwrap();
    stdout_host_std.set_nonblocking(true).unwrap();
    let mut stdout_host = tokio::net::UnixStream::from_std(stdout_host_std).unwrap();

    let stdout = container::pipe::PipeWriter::from_unix_stream(stdout_wasi);

    let mut builder = wasi_common::tokio::WasiCtxBuilder::new();
    builder.stdin(Box::new(stdin_wasi));
    builder.stdout(Box::new(stdout));
    let wasi_ctx = builder.build();

    // WAT module: read 5 bytes from stdin into memory at offset 32,
    // then write those 5 bytes to stdout.
    let wat = r#"
    (module
        (import "wasi_snapshot_preview1" "fd_read"
            (func $fd_read (param i32 i32 i32 i32) (result i32)))
        (import "wasi_snapshot_preview1" "fd_write"
            (func $fd_write (param i32 i32 i32 i32) (result i32)))
        (memory (export "memory") 1)

        (func (export "_start")
            ;; Set up iov for reading at offset 0: { buf_ptr=32, buf_len=5 }
            (i32.store (i32.const 0) (i32.const 32))
            (i32.store (i32.const 4) (i32.const 5))

            ;; fd_read(fd=0, iovs=0, iovs_len=1, nread_ptr=16)
            (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 16))
            drop

            ;; Now set up iov for writing: { buf_ptr=32, buf_len=<nread> }
            ;; nread is at offset 16
            (i32.store (i32.const 0) (i32.const 32))
            (i32.store (i32.const 4) (i32.load (i32.const 16)))

            ;; fd_write(fd=1, iovs=0, iovs_len=1, nwritten_ptr=20)
            (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 20))
            drop
        )
    )
    "#;

    let module = Module::new(&engine, wat).unwrap();

    let engine2 = engine.clone();
    let epoch = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
            engine2.increment_epoch();
        }
    });

    let engine3 = engine.clone();
    let task = tokio::spawn(async move {
        let mut linker = wasmtime::Linker::<WasiCtx>::new(&engine3);
        wasi_common::tokio::add_to_linker(&mut linker, |cx| cx).unwrap();

        let mut store = Store::new(&engine3, wasi_ctx);
        store.epoch_deadline_async_yield_and_update(1);

        linker.module_async(&mut store, "", &module).await.unwrap();
        let func = linker.get_default(&mut store, "").unwrap();
        let typed = func.typed::<(), ()>(&store).unwrap();
        typed.call_async(&mut store, ()).await.unwrap();
    });

    // Write "world" to stdin
    let mut stdin_host = stdin_host;
    stdin_host.write_all(b"world").await.unwrap();

    // Read from stdout
    let mut buf = vec![0u8; 1024];
    let result = tokio::time::timeout(Duration::from_secs(5), stdout_host.read(&mut buf)).await;

    epoch.abort();

    match result {
        Ok(Ok(n)) => {
            let s = String::from_utf8_lossy(&buf[..n]);
            assert_eq!(&buf[..n], b"world", "Expected 'world', got {:?}", s);
        }
        Ok(Err(e)) => panic!("Read error: {e}"),
        Err(_) => panic!("Timeout waiting for stdout output after writing to stdin"),
    }

    task.await.unwrap();
}
