use std::fs;
// use std::sync::mpsc;
// use std::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use clap::{App, Arg, ArgMatches, SubCommand};

pub fn args() -> ArgMatches<'static> {
    let app = App::new("Appname")
        .version("1.0")
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Run")
                .arg(
                    Arg::with_name("file")
                        .long("file")
                        .short("f")
                        .help("file to run")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("loop")
                        .long("loop")
                        .short("n")
                        .help("Repeat run this many times")
                        .required(false)
                        .takes_value(true)
                        .default_value("1"),
                ),
        );

    return app.get_matches();
}

async fn job_feeder(file_name: String, loop_count: i32, mut tx: Sender<String>) {
    let file_contents = fs::read_to_string(file_name).expect("Unable to read file");

    for _i in 0..loop_count {
        let fc = file_contents.clone();
        // We can send a filename instead of file_contents,
        // but ignore that for now.

        // The thread takes ownership over `tx`
        tx.send(fc).await.expect("Unable to send");
    }

    // Docs say sending is a non-blocking operation.
    // how much can it handle without other side consuming?
    println!("Sender queueing finished");
}

fn main() {
    let matches = args();
    // let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (tx, mut rx): (Sender<String>, Receiver<String>) = mpsc::channel(100);

    if let Some(matches) = matches.subcommand_matches("run") {
        let file_name = matches.value_of("file").expect("Unexpected!").to_string(); // clap validates!
        let loop_count = matches
            .value_of("loop")
            .expect("Unexpected! loop is supposed to have a default");

        let loop_count = loop_count.parse::<i32>().expect("invalid number");

        let mut runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();

        let mut total_received = 0;
        runtime.block_on(async {
            let _child = tokio::spawn(job_feeder(file_name, loop_count, tx));

            while let Some(file_contents) = rx.recv().await {
                // Processing file_contents
                // will result in additional tokio::spawn()
                println!("Got {} bytes", file_contents.len());

                // assuming this is safe since there is only *ONE* root task
                total_received += 1
            }
        });

        println!("Got {}", total_received)
    }
}
