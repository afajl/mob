use crate::{command, duration, os};
use anyhow::Result;
use chrono;
use console::Term;
use std::thread;

const FONT_HEIGHT: usize = 7;

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
    fn start(&self, title: &str, duration: chrono::Duration, message: &str) -> Result<()> {
        let mut time_left = duration;
        let second = chrono::Duration::seconds(1);

        let term = Term::stdout();
        term.set_title(title);
        println!("\n{}", title);
        while time_left >= chrono::Duration::zero() {
            let formatted = duration::format(time_left).clock();
            let letters = asci_time(formatted.as_str());

            if time_left != duration {
                term.clear_last_lines(FONT_HEIGHT)?;
            }
            print_ascii(&term, letters)?;
            thread::sleep(second.to_std()?);
            time_left = time_left - second;
        }
        term.clear_last_lines(1)?;

        for cmd in &self.commands {
            let arg = cmd.replace("MESSAGE", message);
            self.sh.run_checked(&["-c", arg.as_str()])?;
        }

        Ok(())
    }
}

fn asci_time(time: &str) -> Vec<&str> {
    time.chars()
        .map(|c| {
            let d = c as usize;
            match d {
                58 => FONT[10], // :
                32 => FONT[11], // <space>
                48..=57 => FONT[d - 48],
                _ => panic!("dont know what to do with '{}' {}", c, d),
            }
        })
        .collect()
}

fn print_ascii(term: &Term, letters: Vec<&str>) -> Result<()> {
    let lines: Vec<String> = (0..FONT_HEIGHT)
        .map(|row| //for row in 0..5 {
        letters
            .iter()
            .map(|l| l.split('\n').nth(row).unwrap())
            .collect::<Vec<&str>>()
            .join(" "))
        .collect();
    for line in lines {
        term.write_line(line.as_str())?;
    }
    Ok(())
}

#[rustfmt::skip]
const FONT: [&str; 12] = [
"
  ██████ 
 ██  ████
 ██ ██ ██
 ████  ██
  ██████ 
",
"
  ██
 ███
  ██
  ██
  ██
 ",
"
 ██████ 
      ██
  █████ 
 ██     
 ███████
",
"
 ██████ 
      ██
  █████ 
      ██
 ██████ 
",
"
 ██   ██
 ██   ██
 ███████
      ██
      ██
",
"
 ███████
 ██     
 ███████
      ██
 ███████
",
"
  ██████ 
 ██      
 ███████ 
 ██    ██
  ██████ 
",
"
 ███████
      ██
     ██ 
    ██  
    ██  
",
"
  █████ 
 ██   ██
  █████ 
 ██   ██
  █████ 
",
"
  █████ 
 ██   ██
  ██████
      ██
  █████ 
",
"
   
 ██
   
 ██
   
",
"
     
     
     
     
     
",
];
