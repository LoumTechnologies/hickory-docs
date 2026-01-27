use wasmtime::{Engine, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::{self, WasiP1Ctx};

fn main() -> wasmtime::Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(
        &engine,
        "crates/c2w/converted_images/alpine/3.23.2.wasm",
    )?;

    let mut linker = wasmtime::Linker::<WasiP1Ctx>::new(&engine);
    p1::add_to_linker_sync(&mut linker, |cx| cx)?;

    let wasi = WasiCtxBuilder::new().inherit_stdio().build_p1();
    let mut store = Store::new(&engine, wasi);

    linker.module(&mut store, "", &module)?;
    linker
        .get_default(&mut store, "")?
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;

    Ok(())
}
