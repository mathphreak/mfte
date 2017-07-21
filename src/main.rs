extern crate termion;
#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;

use termion::event::{Event, Key};
use termion::raw::{RawTerminal, IntoRawMode};
use termion::screen::AlternateScreen;
use termion::input::TermRead;
use std::io::{Write, stdin, stdout};

mod keybinds;
use keybinds::*;

fn render_footer(mut stdout: &mut AlternateScreen<RawTerminal<std::io::Stdout>>, keys: &KeybindTable) {
    use termion::{color, cursor};
    let (screen_width, screen_height) = termion::terminal_size().unwrap();

    let mut x = 1;
    let mut y = screen_height;
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
        x += (key.len() + 1 + action.len() + 1) as u16;
    }
}

fn main() {
    let keys = KeybindTable::from("^Q: Quit\n^X: Quit");
    let stdin = stdin();
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    write!(stdout, "{}{}Press Ctrl-Q to quit", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
    render_footer(&mut stdout, &keys);
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
