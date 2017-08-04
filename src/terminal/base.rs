use std::io::Write;

pub enum Color {
    Reset,
    Black,
    White,
    Grey,
}

// Shamelessly stolen from termios
// which doesn't compile on Win32
// which is why I'm doing all this nonsense in the first place
pub enum Event {
    Key(Key),
    Mouse(MouseEvent),
    Unsupported(Vec<u32>),
}

// Derived from termios, with modifications
// Precedence **must** be Ctrl(Alt(Shift())) in that order
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Shift(Box<Key>),
    Alt(Box<Key>),
    Ctrl(Box<Key>),
    Null,
    Esc,
}

impl Key {
    pub fn is_char(&self) -> bool {
        match self {
            &Key::Char(_) => true,
            _ => false
        }
    }
    
    pub fn is_navigation(&self) -> bool {
        match self {
            &Key::Left | &Key::Right | &Key::Up | &Key::Down |
                &Key::Home | &Key::End | &Key::PageUp | &Key::PageDown => true,
            _ => false
        }
    }
}

// Also termios
pub enum MouseEvent {
    Press(MouseButton, i32, i32),
    Release(i32, i32),
    Hold(i32, i32),
}

// Still termios
pub enum MouseButton {
    Left,
    Right,
    Middle,
    WheelUp,
    WheelDown,
}

// Me again
pub trait TermImpl: Write + Default {
    fn get_size(&self) -> (i32, i32);
    fn goto(&mut self, (i32, i32));
    fn set_color_fg(&mut self, Color);
    fn set_color_bg(&mut self, Color);
    fn clear(&mut self);

    // fn keys(&mut self) -> Iterator<Item = Event>;
}
