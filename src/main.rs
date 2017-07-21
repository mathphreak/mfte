extern crate termion;
#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;

use termion::event::{Event, Key};
use termion::raw::{RawTerminal, IntoRawMode};
use termion::screen::AlternateScreen;
use termion::input::TermRead;
use std::io::{Write, stdin, stdout};
use std::cmp;

mod keybinds;
use keybinds::*;

mod file;
use file::*;

type Terminal = AlternateScreen<RawTerminal<std::io::Stdout>>;

fn render_file(mut stdout: &mut Terminal, file: &File) {
    use termion::{cursor, color};
    let (screen_width, screen_height) = termion::terminal_size().unwrap();

    let mut x = 1;
    let mut y = 1;

    for (line_number, line) in file.lines.iter().enumerate() {
        write!(stdout, "{}{}{:3}{} ",
               color::Fg(color::LightBlack),
               cursor::Goto(x, y),
               (line_number + 1),
               color::Fg(color::Reset)).unwrap();
        x += 4;
        let mut line_start = 0;
        let mut line_end = cmp::min(line.len(), (screen_width - x) as usize);
        write!(stdout, "{}{}", cursor::Goto(x, y), &line[line_start..line_end]).unwrap();
        while line_end < line.len() {
            x = 5;
            y += 1;
            line_start = line_end;
            line_end += cmp::min(line.len() - line_end, (screen_width - x) as usize);
            write!(stdout, "{}{}", cursor::Goto(x, y), &line[line_start..line_end]).unwrap();
        }
        y += 1;
        x = 1;
    }
}

fn render_footer(mut stdout: &mut Terminal, keys: &KeybindTable) {
    use termion::{color, cursor};
    let (screen_width, screen_height) = termion::terminal_size().unwrap();

    let mut x = 1;
    let mut y = screen_height - 1;
    for (key, action) in keys.entries() {
        write!(stdout, "{}{}{}{}",
               cursor::Goto(x, y),
               color::Bg(color::White),
               color::Fg(color::Black),
               key).unwrap();
        write!(stdout, "{}{} {}",
               color::Bg(color::Reset),
               color::Fg(color::Reset),
               action).unwrap();
        x += 16;
        if x > screen_width {
            x = 1;
            y -= 1;
        }
    }
}

fn main() {
    let keys = KeybindTable::default();
    let stdin = stdin();
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    write!(stdout, "{}", termion::clear::All).unwrap();
    render_footer(&mut stdout, &keys);
    write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
    let file = File::open("README.md");
    render_file(&mut stdout, &file);
    stdout.flush().unwrap();
    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(Key::Ctrl(k)) => {
                match keys.lookup(Key::Ctrl(k)) {
                    Some(Command::Quit) => break,
                    _ => ()
                }
            },
            _ => {}
        }
        stdout.flush().unwrap();
    }
}
