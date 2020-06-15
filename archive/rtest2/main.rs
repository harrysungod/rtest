use deno_core::CoreIsolate;
use deno_core::StartupData;
use tokio;

fn main() {
    let startup_data = StartupData::Script(deno_core::Script {
        // source: "Deno.core.print(\"Hello world\");",
        source: include_str!("simple.js"),
        filename: "helloworld.js",
    });

    let mut core_isolate = CoreIsolate::new(startup_data, false);

    /*
    let result = core_isolate
        .execute("something.js", "throw \"test2\";")
        .expect("Execute failed");
    */

    let mut runtime = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    runtime
        .block_on(core_isolate)
        .expect("unexpected isolate error");
}
