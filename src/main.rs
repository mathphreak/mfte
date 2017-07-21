extern crate termion;

use termion::event::{Event, Key};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use std::io::{Write, stdin, stdout};

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}{}Press Ctrl-Q to quit", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
    stdout.flush().unwrap();
    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(Key::Ctrl('q')) => break,
            _ => {}
        }
        stdout.flush().unwrap();
    }
    write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
}
