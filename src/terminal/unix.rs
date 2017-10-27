extern crate termion;

use self::termion::event;
use self::termion::raw::{RawTerminal, IntoRawMode};
use self::termion::screen::AlternateScreen;
use self::termion::input::TermRead;
use self::termion::cursor;
use self::termion::color;
use std::io::{self, Write};
use std::iter;

use super::base::*;

macro_rules! decode_color {
    ($x:expr, $c:expr) => {{
        match $c {
            Color::Reset => format!("{}", $x(color::Reset)),
            Color::Black => format!("{}", $x(color::Black)),
            Color::White => format!("{}", $x(color::White)),
            Color::Grey => format!("{}", $x(color::LightBlack)),
        }
    }}
}

impl From<event::Key> for Key {
    fn from(k: event::Key) -> Key {
        match k {
            event::Key::Backspace => Key::Backspace,
            event::Key::Left => Key::Left,
            event::Key::Right => Key::Right,
            event::Key::Up => Key::Up,
            event::Key::Down => Key::Down,
            event::Key::Home => Key::Home,
            event::Key::End => Key::End,
            event::Key::PageUp => Key::PageUp,
            event::Key::PageDown => Key::PageDown,
            event::Key::Delete => Key::Delete,
            event::Key::Insert => Key::Insert,
            event::Key::F(n) => Key::F(n),
            event::Key::Char(c) => Key::Char(c),
            event::Key::Alt(c) => Key::Alt(Box::new(Key::Char(c))),
            event::Key::Ctrl(c) => Key::Ctrl(Box::new(Key::Char(c))),
            event::Key::Null => Key::Null,
            event::Key::Esc => Key::Esc,
            event::Key::__IsNotComplete => panic!("got a weird key from termios"),
        }
    }
}

impl From<event::MouseEvent> for MouseEvent {
    fn from(e: event::MouseEvent) -> MouseEvent {
        match e {
            event::MouseEvent::Press(b, x, y) => MouseEvent::Press(MouseButton::from(b), x as i32, y as i32),
            event::MouseEvent::Release(x, y) => MouseEvent::Release(x as i32, y as i32),
            event::MouseEvent::Hold(x, y) => MouseEvent::Hold(x as i32, y as i32),
        }
    }
}

impl From<event::MouseButton> for MouseButton {
    fn from(b: event::MouseButton) -> MouseButton {
        match b {
            event::MouseButton::Left => MouseButton::Left,
            event::MouseButton::Right => MouseButton::Right,
            event::MouseButton::Middle => MouseButton::Middle,
            event::MouseButton::WheelUp => MouseButton::WheelUp,
            event::MouseButton::WheelDown => MouseButton::WheelDown,
        }
    }
}

impl From<event::Event> for Event {
    fn from(e: event::Event) -> Event {
        match e {
            event::Event::Key(k) => Event::Key(k.into()),
            event::Event::Mouse(m) => Event::Mouse(m.into()),
            event::Event::Unsupported(v) => {
                let vals = v.iter().map(|x| *x as u32).collect();
                if vals == [27, 91, 49, 59, 50, 65] { // <Esc>[1;2A
                    Event::Key(Key::Shift(Box::new(Key::Up)))
                } else if vals == [27, 91, 49, 59, 50, 66] { // <Esc>[1;2B
                    Event::Key(Key::Shift(Box::new(Key::Down)))
                } else if vals == [27, 91, 49, 59, 50, 67] { // <Esc>[1;2C
                    Event::Key(Key::Shift(Box::new(Key::Right)))
                } else if vals == [27, 91, 49, 59, 50, 68] { // <Esc>[1;2D
                    Event::Key(Key::Shift(Box::new(Key::Left)))
                } else {
                    Event::Unsupported(vals)
                }
            }
        }
    }
}

pub struct Terminal {
    out: AlternateScreen<RawTerminal<io::Stdout>>,
}

impl Terminal {
    pub fn keys(&mut self) -> iter::Map<termion::input::Events<io::Stdin>, fn(io::Result<event::Event>) -> Event> {
        fn extract(e: io::Result<event::Event>) -> Event { Event::from(e.unwrap()) }
        io::stdin().events().map(extract)
    }
}

impl Write for Terminal {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.out.write(data)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

impl Default for Terminal {
    fn default() -> Terminal {
        Terminal {
            out: AlternateScreen::from(io::stdout().into_raw_mode().unwrap()),
        }
    }
}

impl TermImpl for Terminal {
    fn get_size(&self) -> (i32, i32) {
        let (x, y) = termion::terminal_size().unwrap();
        (x as i32, y as i32)
    }

    fn goto(&mut self, (x, y): (i32, i32)) {
        write!(self.out, "{}", cursor::Goto(x as u16, y as u16)).unwrap();
    }

    fn set_color_fg(&mut self, c: Color) {
        write!(self.out, "{}", decode_color!(color::Fg, c)).unwrap();
    }

    fn set_color_bg(&mut self, c: Color) {
        write!(self.out, "{}", decode_color!(color::Bg, c)).unwrap();
    }

    fn clear(&mut self) {
        write!(self.out, "{}", termion::clear::All).unwrap();
    }
}
