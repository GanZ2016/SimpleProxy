//Reference https://doc.rust-lang.org/log/log/index.html
use std::vec::Vec;
use std::collections::vec_deque::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use log;
use log::{LogRecord, LogLevel, LogMetadata, SetLoggerError};

struct ChannelLogger {
    level: LogLevel,
}

impl log::Log for ChannelLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(),record.args());

        }
    }

}


pub fn init(level: LogLevel, log_path: String) -> Result<(), SetLoggerError> {
    log::set_logger(|max_level| {
        max_level.set(log::LogLevelFilter::Info);
        Box::new(ChannelLogger{ level: level})
    })

}
