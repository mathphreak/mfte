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
    Unsupported(Vec<u8>),
}

// Still stolen from termios
#[derive(PartialEq, Eq, Hash, Debug)]
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
    Alt(char),
    Ctrl(char),
    Null,
    Esc,
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
