use std::env;

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

fn main() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(
        env::args()
            .find(|a| a == "-D")
            .map(|_| log::LevelFilter::Debug)
            .unwrap_or(log::LevelFilter::Warn),
    );
}
