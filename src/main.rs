extern crate termion;
#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;

use termion::event::{Event, Key};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use std::io::{Write, stdin, stdout};

mod keybinds;
use keybinds::*;

fn main() {
    let keys = KeybindTable::from("^Q: Quit");
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}{}Press Ctrl-Q to quit", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
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
    write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
}
