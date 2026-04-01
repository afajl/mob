extern crate log;
use console::{Emoji, style};
use env_logger::Env;
use log::Level;

pub fn init(level: &str) {
    let mut builder = env_logger::Builder::from_env(Env::default().default_filter_or(level));

    builder.format(|f, record| {
        use std::io::Write;

        match record.level() {
            Level::Trace => writeln!(
                f,
                " {} {}",
                Emoji("🔍", "TRACE"),
                style(record.args()).magenta()
            ),
            Level::Debug => writeln!(
                f,
                " {} {}",
                Emoji("›", "DEBUG"),
                style(record.args()).blue()
            ),
            Level::Info => writeln!(
                f,
                " {} {}",
                Emoji(">", "INFO"),
                style(record.args()).green()
            ),
            Level::Warn => writeln!(
                f,
                " {} {}",
                Emoji("⚠️", "WARN"),
                style(record.args()).yellow()
            ),
            Level::Error => writeln!(
                f,
                " {} {}",
                Emoji("⚡", "ERROR"),
                style(record.args()).red()
            ),
        }
    });

    builder.init();
}
