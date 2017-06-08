//Reference https://doc.rust-lang.org/log/log/index.html
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


pub fn init(level: LogLevel) -> Result<(), SetLoggerError> {
    log::set_logger(|max_level| {
        max_level.set(log::LogLevelFilter::Info);
        Box::new(ChannelLogger{ level: level})
    })

}
