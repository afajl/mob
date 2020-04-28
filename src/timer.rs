use crate::{command, os};
use anyhow::Result;
use chrono;
use console::Term;
use std::thread;

pub trait Timer {
    fn start(&self, title: &str, duration: chrono::Duration, message: &str) -> Result<()>;
}

pub struct ConsoleTimer<'a> {
    sh: command::Command<'a>,
    commands: Vec<String>,
}

impl<'a> ConsoleTimer<'a> {
    pub fn new(commands: Vec<String>) -> ConsoleTimer<'a> {
        ConsoleTimer {
            sh: command::Command::new(os::command("sh")),
            commands,
        }
    }
}

impl<'a> Timer for ConsoleTimer<'a> {
    fn start(&self, title: &str, mut duration: chrono::Duration, message: &str) -> Result<()> {
        let second = chrono::Duration::seconds(1);

        let term = Term::stdout();
        term.set_title(title);
        term.write_line("")?;
        while duration > chrono::Duration::zero() {
            //term.clear_last_lines(1)?;
            term.clear_last_lines(1)?;
            //term.move_cursor_up(1)?;
            let line = format!("{}: {}", title, format_duration(duration));
            term.write_line(line.as_str())?;
            thread::sleep(second.to_std()?);
            duration = duration - second;
        }
        term.clear_last_lines(1)?;

        for cmd in &self.commands {
            let arg = cmd.replace("MESSAGE", message);
            self.sh.run_checked(&["-c", arg.as_str()])?;
        }

        Ok(())
    }
}

fn format_duration(duration: chrono::Duration) -> String {
    let h = duration.num_hours();
    let m = duration.num_minutes() - h * 60;
    let s = duration.num_seconds() - m * 60;

    if duration.num_hours() > 0 {
        format!("{:>2}:{:0>2}", h, m)
    } else if duration.num_minutes() > 0 {
        format!("{:>2}:{:0>2}", m, s)
    } else {
        format!("{:>2}", s)
    }
}

// const FONT: [&str; 11] = [
//     "
//  ██████
// ██  ████
// ██ ██ ██
// ████  ██
//  ██████
// ",
//     "
//  ██
// ███
//  ██
//  ██
//  ██
// ",
//     "
// ██████
//      ██
//  █████
// ██
// ███████
// ",
//     "
// ██████
//      ██
//  █████
//      ██
// ██████
// ",
//     "
// ██   ██
// ██   ██
// ███████
//      ██
//      ██
// ",
//     "
// ███████
// ██
// ███████
//      ██
// ███████
// ",
//     "
//  ██████
// ██
// ███████
// ██    ██
//  ██████
// ",
//     "
// ███████
//      ██
//     ██
//    ██
//    ██
// ",
//     "
//  █████
// ██   ██
//  █████
// ██   ██
//  █████
// ",
//     "
//  █████
// ██   ██
//  ██████
//      ██
//  █████
// ",
//     "

// ██
//
// ██
//
// ",
// ];
