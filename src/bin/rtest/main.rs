use deno_core::CoreIsolate;
use deno_core::StartupData;
use tokio;

fn main() {
    let startup_data = StartupData::Script(deno_core::Script {
        source: "console.log(\"hello world\");",
        filename: "helloworld.js",
    });

    let core_isolate = CoreIsolate::new(startup_data, false);

    let mut runtime = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    runtime
        .block_on(core_isolate)
        .expect("unexpected isolate error");
}
