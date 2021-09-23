use crate::duration;
use anyhow::Result;
use chrono;
use console::Term;
use std::thread;

const FONT_HEIGHT: usize = 7;

pub fn start(title: &str, duration: chrono::Duration) -> Result<()> {
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

    Ok(())
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
