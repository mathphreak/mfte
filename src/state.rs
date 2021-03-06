extern crate clipboard;
use self::clipboard::{ClipboardProvider, ClipboardContext};

use super::keybinds::*;
use super::file::*;

pub struct OneLinerState {
    pub command: Command,
    pub label: &'static str,
    pub file: File,
}

impl From<Command> for OneLinerState {
    fn from(c: Command) -> OneLinerState {
        let label = match c {
            Command::Quit | Command::CloseFile | Command::Refresh |
                Command::Cut | Command::Copy | Command::Paste |
                Command::Undo | Command::Redo |
                Command::NewTab => "",
            Command::OpenFile => "Open file:",
            Command::SaveFile => "Save file:",
            Command::Goto => "Jump to line[:col]:",
            Command::Find => "Find text:",
            Command::FindReplace => "AAAAAAAAAA",
        };
        OneLinerState {
            command: c,
            label: label,
            file: File::empty(),
        }
    }
}

impl OneLinerState {
    pub fn value(&self) -> &String {
        &self.file.lines[0]
    }

    pub fn tab(&mut self) {
        use std::path::PathBuf;
        let mut path = PathBuf::from(self.value());
        if path.file_name().is_none() {
            path = PathBuf::from(".");
        }
        let mut fragment = String::from("");
        if !path.is_dir() {
            fragment = path.file_name().unwrap().to_os_string().into_string().unwrap();
            path.pop();
        }
        let results: Vec<String> = path.read_dir().expect("did not get a dir").filter_map(|e| e.ok().and_then(|e| {
            let name = e.file_name().into_string().unwrap();
            if name.starts_with(&fragment) {
                Some(name)
            } else {
                None
            }
        })).collect();
        if results.len() > 0 {
            let first = results[0].clone();
            let shared = results.iter().fold(first, |a, b| {
                a.chars().zip(b.chars()).take_while(|a| a.0 == a.1).map(|a| a.0).collect::<String>()
            });
            if shared.len() > 0 {
                path.push(shared.clone());
                if path.is_dir() && results.len() == 1 {
                    path.push("");
                }
                self.file.lines[0] = path.into_os_string().into_string().unwrap();
                self.file.move_cursor_end((9001, 9001));
                self.file.display_dirty = true;
            }
        }
    }
}

pub struct EditorState {
    pub keys: KeybindTable,
    pub files: Vec<File>,
    pub one_liners: Vec<Option<OneLinerState>>,
    pub active_file: usize,
}

/* Man, I hate Rust sometimes.
 * This used to look like
 *     match self.one_liner_mut() {
 *         &mut Some(ref mut ols) => ols.file.$func(dim),
 *         &mut None => self.active_file_mut().$func(dim)
 *     }
 * but both _mut() calls borrow self mutably, and apparently that's not legal.
 * Even though I'm done with the borrow I'm matching on by the time I get to that arm.
 *
 * Too much research later, apparently they're working on it: https://github.com/rust-lang/rfcs/issues/811
 */

macro_rules! split_func {
    ($func:ident -> $ret:ty) => {
        pub fn $func(&mut self, dim: (i32, i32)) -> $ret {
            match self.one_liner_mut() {
                &mut Some(ref mut ols) => return ols.file.$func(dim),
                _ => ()
            }
            self.active_file_mut().$func(dim)
        }
    };
    ($func:ident) => {split_func!($func -> ());};
}

macro_rules! restrict_func {
    ($func:ident -> $ret:ty, $default:expr) => {
        pub fn $func(&mut self, dim: (i32, i32)) -> $ret {
            match self.one_liner() {
                &Some(_) => $default,
                &None => self.active_file_mut().$func(dim)
            }
        }
    };
    ($func:ident) => {restrict_func!($func -> (), ());};
}

impl EditorState {
    pub fn one_liner_active(&self) -> bool {
        self.one_liner().is_some()
    }

    pub fn consume_one_liner(&mut self) -> Option<(Command, String)> {
        let result = self.one_liner_mut().take().map(|ol| (ol.command.clone(), (*ol.value()).clone()));
        result
    }

    pub fn lineno_chars(&self) -> i32 {
        self.active_file().lineno_chars()
    }

    pub fn active_file(&self) -> &File {
        &self.files[self.active_file]
    }

    pub fn active_file_mut(&mut self) -> &mut File {
        &mut self.files[self.active_file]
    }

    pub fn one_liner(&self) -> &Option<OneLinerState> {
        &self.one_liners[self.active_file]
    }

    pub fn one_liner_mut(&mut self) -> &mut Option<OneLinerState> {
        &mut self.one_liners[self.active_file]
    }

    pub fn set_one_liner(&mut self, ol: OneLinerState) {
        self.one_liners[self.active_file] = Some(ol);
    }

    pub fn cursor(&self, dim: (i32, i32)) -> (i32, i32) {
        match self.one_liner() {
            &Some(ref ols) => {
                let cursor = ols.file.cursor(dim);
                (cursor.x + ols.label.len() as i32 + 1, dim.1 + 1)
            },
            &None => {
                let cursor = self.active_file().cursor(dim);
                (cursor.x + self.active_file().lineno_chars() + 1, cursor.y)
            }
        }
    }

    pub fn debug(&self, dim: (i32, i32)) -> String {
        self.active_file().debug(dim)
    }

    pub fn chunked_text(&self, dim: (i32, i32)) -> Vec<(Option<u16>, Vec<TextChunk>)> {
        self.active_file().chunked_text(dim)
    }

    pub fn refresh(&mut self, dim: (i32, i32)) {
        self.active_file_mut().refresh(dim)
    }

    pub fn new_tab(&mut self) {
        self.active_file += 1;
        self.files.insert(self.active_file, File::empty());
        self.one_liners.insert(self.active_file, None);
    }

    pub fn close_tab(&mut self) {
        self.files.remove(self.active_file);
        self.one_liners.remove(self.active_file);
        if self.files.len() > 0 {
            self.active_file %= self.files.len();
        }
    }

    pub fn next_tab(&mut self) {
        self.active_file = (self.active_file + 1) % self.files.len();
    }

    pub fn cut(&mut self, dim: (i32, i32)) {
        if self.active_file().has_selection() {
            self.copy(dim);
            self.delete(dim);
        }
    }

    pub fn copy(&mut self, _: (i32, i32)) {
        if self.active_file().has_selection() {
            let selection = self.active_file().selected_text();
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            ctx.set_contents(selection).unwrap();
        }
    }

    pub fn paste(&mut self, dim: (i32, i32)) {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        let clipboard = ctx.get_contents().unwrap();
        for c in clipboard.chars() {
            if c == '\n' {
                self.insert_newline(dim, false);
            } else {
                self.insert(dim, c);
            }
        }
    }

    pub fn select(&mut self) {
        match self.one_liner_mut() {
            &mut Some(ref mut ols) => return ols.file.select(),
            _ => ()
        }
        self.active_file_mut().select();
    }

    pub fn deselect(&mut self) {
        match self.one_liner_mut() {
            &mut Some(ref mut ols) => return ols.file.deselect(),
            _ => ()
        }
        self.active_file_mut().deselect();
    }

    pub fn display_dirty(&self) -> bool {
        match self.one_liner() {
            &Some(ref ols) => ols.file.display_dirty,
            _ => self.active_file().display_dirty
        }
    }

    pub fn clean_display(&mut self) {
        match self.one_liner_mut() {
            &mut Some(ref mut ols) => {
                ols.file.display_dirty = false;
                return;
            },
            _ => ()
        }
        self.active_file_mut().display_dirty = false;
    }

    split_func!(move_cursor_left);
    split_func!(move_cursor_right);
    restrict_func!(move_cursor_up);
    restrict_func!(move_cursor_down);
    split_func!(move_cursor_home);
    split_func!(move_cursor_end);
    restrict_func!(page_up);
    restrict_func!(page_down);
    restrict_func!(scroll_up);
    restrict_func!(scroll_down);

    pub fn move_cursor_to(&mut self, dim: (i32, i32), dest: (i32, i32)) {
        let mut last_cursor = (-1, -1);
        while self.cursor(dim).1 < dest.1 && self.cursor(dim) != last_cursor {
            last_cursor = self.cursor(dim);
            self.move_cursor_down(dim);
        }
        while self.cursor(dim).1 > dest.1 && self.cursor(dim) != last_cursor {
            last_cursor = self.cursor(dim);
            self.move_cursor_up(dim);
        }
        self.move_cursor_end(dim);
        last_cursor = (-1, -1);
        while dest.0 < self.cursor(dim).0 && self.cursor(dim) != last_cursor {
            last_cursor = self.cursor(dim);
            self.move_cursor_left(dim);
        }
    }

    pub fn goto(&mut self, dim: (i32, i32), target: &str) {
        let colon_idx = target.find(':');
        let (row, col) = match colon_idx {
            Some(n) => {
                let (row_str, col_str) = target.split_at(n);
                let (_, col_str) = col_str.split_at(1);
                (i32::from_str_radix(row_str, 10).unwrap(), i32::from_str_radix(col_str, 10).unwrap())
            },
            None => {
                (i32::from_str_radix(target, 10).unwrap(), self.active_file().caret.x)
            }
        };
        let row = if row < 1 {
            1
        } else if row > self.active_file().lines.len() as i32 {
            self.active_file().lines.len() as i32
        } else {
            row
        };
        self.active_file_mut().goto(dim, (row, col));
    }

    pub fn insert_newline(&mut self, dim: (i32, i32), indent: bool) {
        match self.one_liner() {
            &Some(_) => panic!("Can't insert newline in one liner! That's the point!"),
            &None => self.active_file_mut().insert_newline(dim, indent)
        }
    }

    split_func!(delete);
    split_func!(backspace);
    restrict_func!(tab);

    pub fn insert(&mut self, dim: (i32, i32), c: char) {
        match self.one_liner_mut() {
            &mut Some(ref mut ols) => return ols.file.insert(dim, c),
            _ => ()
        };
        self.active_file_mut().insert(dim, c)
    }

    pub fn save_file(&mut self, path: &str) {
        self.active_file_mut().save(path);
    }

    pub fn open_file(&mut self, path: &str) {
        self.files[self.active_file] = File::open(path);
    }
}
