extern crate log;
use env_logger::{
    self,
    fmt::{Color, Style},
    Env,
};
use log::Level;

pub fn init(level: &str) {
    let mut builder = env_logger::from_env(Env::default().default_filter_or(level));

    builder.format(|f, record| {
        use std::io::Write;

        let mut style = f.style();
        let emoji = level_style(&mut style, record.level());

        writeln!(f, " {}  {}", emoji, style.value(record.args()))
    });

    builder.init()
}

fn level_style(style: &mut Style, level: Level) -> &'static str {
    match level {
        Level::Trace => {
            style.set_color(Color::Magenta);
            "🔍"
        }
        Level::Debug => {
            style.set_color(Color::Blue);
            "›"
        }
        Level::Info => {
            style.set_color(Color::Green);
            ">"
        }
        Level::Warn => {
            style.set_color(Color::Yellow);
            "⚠️"
        }
        Level::Error => {
            style.set_color(Color::Red);
            "⚡"
        }
    }
}
