extern crate winapi;
extern crate kernel32;

use std::io::{self, Write};
use std::ptr;

use self::winapi::winnt;
use self::winapi::winbase;
use self::winapi::wincon;

use super::base::*;

pub struct Terminal {
    stdin: winnt::HANDLE,
    stdout: winnt::HANDLE,
    orig_mode: winapi::DWORD,
    char_attr_bg: winapi::WORD,
    char_attr_fg: winapi::WORD,
}

pub struct TerminalKeyStream {
    stdin: winnt::HANDLE,
}

fn get_key(e: wincon::KEY_EVENT_RECORD) -> Option<Key> {
    if e.bKeyDown == 0 {
        return None;
    }
    let key_mod = |mut key| {
        let alt_flags = wincon::LEFT_ALT_PRESSED | wincon::RIGHT_ALT_PRESSED;
        let ctrl_flags = wincon::LEFT_CTRL_PRESSED | wincon::RIGHT_CTRL_PRESSED;
        if e.dwControlKeyState & wincon::SHIFT_PRESSED != 0 {
            key = Key::Shift(Box::new(key));
        }
        if e.dwControlKeyState & alt_flags != 0 {
            key = Key::Alt(Box::new(key));
        }
        if e.dwControlKeyState & ctrl_flags != 0 {
            key = Key::Ctrl(Box::new(key));
        }
        Some(key)
    };
    let char_mod = |c: char| {
        if e.dwControlKeyState & wincon::SHIFT_PRESSED == 0 {
            Key::Char(c.to_lowercase().next().unwrap())
        } else {
            Key::Char(c.to_uppercase().next().unwrap())
        }
    };
    match e.wVirtualKeyCode as i32 {
        winapi::VK_BACK => return key_mod(Key::Backspace),
        winapi::VK_TAB => return key_mod(char_mod('\t')),
        winapi::VK_RETURN => return key_mod(Key::Char('\n')),
        winapi::VK_ESCAPE => return key_mod(Key::Esc),
        winapi::VK_PRIOR => return key_mod(Key::PageUp),
        winapi::VK_NEXT => return key_mod(Key::PageDown),
        winapi::VK_END => return key_mod(Key::End),
        winapi::VK_HOME => return key_mod(Key::Home),
        winapi::VK_LEFT => return key_mod(Key::Left),
        winapi::VK_UP => return key_mod(Key::Up),
        winapi::VK_RIGHT => return key_mod(Key::Right),
        winapi::VK_DOWN => return key_mod(Key::Down),
        winapi::VK_INSERT => return key_mod(Key::Insert),
        winapi::VK_DELETE => return key_mod(Key::Delete),
        0x41...0x5A => return key_mod(char_mod(e.wVirtualKeyCode as u8 as char)),
        winapi::VK_F1...winapi::VK_F24 => {
            return key_mod(Key::F((e.wVirtualKeyCode as i32 - winapi::VK_F1 + 1) as u8))
        },
        _ => (),
    };
    if e.UnicodeChar == 0 {
        Some(Key::Null)
    } else {
        key_mod(char_mod(e.UnicodeChar as u8 as char))
    }
}

fn get_mouse_event(e: wincon::MOUSE_EVENT_RECORD) -> Option<MouseEvent> {
    let pos = e.dwMousePosition;
    let button = match e.dwEventFlags {
        0 => Some(match e.dwButtonState {
            0 => {
                return Some(MouseEvent::Release(pos.X as i32 + 1, pos.Y as i32 + 1));
            },
            wincon::FROM_LEFT_1ST_BUTTON_PRESSED => MouseButton::Left,
            wincon::RIGHTMOST_BUTTON_PRESSED => MouseButton::Right,
            _ => MouseButton::Middle
        }),
        wincon::MOUSE_WHEELED => Some({
            // Win32 API docs say "if high word is positive, scroll up"
            if e.dwButtonState.leading_zeros() > 0 {
                MouseButton::WheelUp
            } else {
                MouseButton::WheelDown
            }
        }),
        wincon::MOUSE_MOVED => {
            return Some(MouseEvent::Hold(pos.X as i32 + 1, pos.Y as i32 + 1));
        },
        _ => None
    };
    if let Some(b) = button {
        Some(MouseEvent::Press(b, pos.X as i32 + 1, pos.Y as i32 + 1))
    } else {
        None
    }
}

impl Iterator for TerminalKeyStream {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        loop {
            let mut buf = wincon::INPUT_RECORD {
                EventType: 0,
                Event: [0, 0, 0, 0]
            };
            let mut count: winapi::DWORD = 0;
            let processed = unsafe { kernel32::ReadConsoleInputW(
                self.stdin,
                &mut buf,
                1,
                &mut count
            ) } != 0;
            if !processed {
                eprintln!("Win32 hates your guts. {}", io::Error::last_os_error());
                // Welp, I guess Win32 hates your guts
                return None;
            }
            if count == 0 {
                continue;
            }
            match buf.EventType {
                wincon::KEY_EVENT => {
                    if let Some(key) = get_key(unsafe { buf.KeyEvent().clone() }) {
                        return Some(Event::Key(key));
                    }
                },
                wincon::MOUSE_EVENT => {
                    if let Some(e) = get_mouse_event(unsafe { buf.MouseEvent().clone() }) {
                        return Some(Event::Mouse(e));
                    }
                },
                _ => return Some(Event::Unsupported(buf.Event.to_vec()))
            }
        }
    }
}

impl Terminal {
    pub fn keys(&mut self) -> TerminalKeyStream {
        TerminalKeyStream {
            stdin: self.stdin
        }
    }
}

macro_rules! empty_coord {
    () => { wincon::COORD { X: 0, Y: 0 } }
}

macro_rules! empty_csbi {
    () => { wincon::CONSOLE_SCREEN_BUFFER_INFO {
        dwSize: empty_coord!(),
        dwCursorPosition: empty_coord!(),
        wAttributes: 0,
        srWindow: wincon::SMALL_RECT {
            Left: 0,
            Top: 0,
            Right: 0,
            Bottom: 0
        },
        dwMaximumWindowSize: empty_coord!()
    }}
}

impl Write for Terminal {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let chars_valid = unsafe { kernel32::WriteConsoleA(
            self.stdout,
            data.as_ptr() as *const winnt::VOID,
            data.len() as u32,
            ptr::null::<winapi::DWORD>() as winapi::LPDWORD,
            ptr::null::<winapi::VOID>() as winapi::LPVOID
        ) } != 0;
        if chars_valid {
            Ok(data.len())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, io::Error::last_os_error()))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        // TODO figure out if this is actually impossible on Win32
        Ok(())
    }
}

impl Default for Terminal {
    fn default() -> Terminal {
        let mut result = Terminal {
            // PSA: don't mix these up. That causes problems.
            stdin: unsafe { kernel32::GetStdHandle(winbase::STD_INPUT_HANDLE) },
            stdout: unsafe { kernel32::GetStdHandle(winbase::STD_OUTPUT_HANDLE) },
            orig_mode: 0,
            char_attr_bg: 0,
            char_attr_fg: (wincon::FOREGROUND_RED |
                wincon::FOREGROUND_GREEN | wincon::FOREGROUND_BLUE) as winapi::WORD,
        };
        unsafe {
            kernel32::GetConsoleMode(result.stdin, &mut result.orig_mode);
            kernel32::SetConsoleMode(result.stdin, wincon::ENABLE_MOUSE_INPUT |
                                     wincon::ENABLE_EXTENDED_FLAGS);
        };
        result
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.clear();
        unsafe { kernel32::SetConsoleMode(self.stdout, self.orig_mode); };
    }
}

impl TermImpl for Terminal {
    fn get_size(&self) -> (i32, i32) {
        let mut buf = empty_csbi!();
        unsafe { kernel32::GetConsoleScreenBufferInfo(self.stdout, &mut buf); };
        let win = buf.srWindow;
        ((win.Right - win.Left + 1) as i32, (win.Bottom - win.Top + 1) as i32)
    }

    fn goto(&mut self, (x, y): (i32, i32)) {
        let pos = wincon::COORD {
            X: (x - 1) as i16,
            Y: (y - 1) as i16
        };
        unsafe { kernel32::SetConsoleCursorPosition(self.stdout, pos); };
    }

    fn set_color_fg(&mut self, c: Color) {
        use self::wincon::*;
        self.char_attr_fg = match c {
            Color::Reset => FOREGROUND_RED | FOREGROUND_GREEN | FOREGROUND_BLUE,
            Color::Black => 0,
            Color::White => FOREGROUND_RED | FOREGROUND_GREEN | FOREGROUND_BLUE,
            Color::Grey => FOREGROUND_INTENSITY,
        } as winapi::WORD;
        unsafe {
            kernel32::SetConsoleTextAttribute(self.stdout, self.char_attr_bg | self.char_attr_fg);
        }
    }

    fn set_color_bg(&mut self, c: Color) {
        use self::wincon::*;
        self.char_attr_bg = match c {
            Color::Reset => 0,
            Color::Black => 0,
            Color::White => BACKGROUND_RED | BACKGROUND_GREEN | BACKGROUND_BLUE,
            Color::Grey => BACKGROUND_INTENSITY,
        } as winapi::WORD;
        unsafe {
            kernel32::SetConsoleTextAttribute(self.stdout, self.char_attr_bg | self.char_attr_fg);
        }
    }

    fn clear(&mut self) {
        // Somebody at Microsoft was asleep at the wheel and didn't write
        // an API call for this. So instead we get to do all this nonsense
        // (from https://support.microsoft.com/en-us/help/99261/how-to-performing-clear-screen-cls-in-a-console-application)
        let coord_screen = wincon::COORD { X: 0, Y: 0 };
        let mut buf = empty_csbi!();

        unsafe {
            kernel32::GetConsoleScreenBufferInfo(self.stdout, &mut buf);
        }

        let console_size = (buf.dwSize.X as u32) * (buf.dwSize.Y as u32);

        let mut n: winapi::DWORD = 0;

        unsafe {
            kernel32::FillConsoleOutputCharacterA(
                self.stdout, ' ' as i8, console_size, coord_screen,
                &mut n
            );

            kernel32::FillConsoleOutputAttribute(
                self.stdout, buf.wAttributes, console_size, coord_screen,
                &mut n
            );

            kernel32::SetConsoleCursorPosition(self.stdout, coord_screen);
        }
    }
}
