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
                Command::Undo | Command::Redo => "",
            Command::OpenFile => "Open file:",
            Command::SaveFile => "Save file:",
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
        // TODO figure out if any of this is a good idea
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
            let mut name = e.file_name().into_string().unwrap();
            if name.starts_with(&fragment) {
                Some(name)
            } else {
                None
            }
        })).collect();
        if results.len() == 1 {
            path.push(results[0].clone());
            if path.is_dir() {
                path.push("");
            }
            self.file.lines[0] = path.into_os_string().into_string().unwrap();
            self.file.move_cursor_end((9001, 9001));
        }
    }
}

pub struct EditorState {
    pub keys: KeybindTable,
    pub file: File,
    pub one_liner: Option<OneLinerState>,
}

macro_rules! split_func {
    ($func:ident -> $ret:ty) => {
        pub fn $func(&mut self, dim: (i32, i32)) -> $ret {
            match self.one_liner {
                Some(ref mut ols) => ols.file.$func(dim),
                None => self.file.$func(dim)
            }
        }
    };
    ($func:ident) => {split_func!($func -> ());};
}

macro_rules! restrict_func {
    ($func:ident -> $ret:ty, $default:expr) => {
        pub fn $func(&mut self, dim: (i32, i32)) -> $ret {
            match self.one_liner {
                Some(_) => $default,
                None => self.file.$func(dim)
            }
        }
    };
    ($func:ident) => {restrict_func!($func -> (), ());};
}

impl EditorState {
    pub fn lines_len(&self) -> usize {
        self.file.lines.len()
    }

    pub fn one_liner_active(&self) -> bool {
        self.one_liner.is_some()
    }

    pub fn lineno_chars(&self) -> i32 {
        self.file.lineno_chars()
    }

    pub fn cursor(&self, dim: (i32, i32)) -> (i32, i32) {
        match self.one_liner {
            Some(ref ols) => {
                let cursor = ols.file.cursor(dim);
                (cursor.x + ols.label.len() as i32 + 1, dim.1 + 1)
            },
            None => {
                let cursor = self.file.cursor(dim);
                (cursor.x + self.file.lineno_chars() + 1, cursor.y)
            }
        }
    }

    pub fn wrapped_lines(&self, dim: (i32, i32)) -> Vec<(Option<u16>, String)> {
        self.file.wrapped_lines(dim)
    }

    pub fn refresh(&mut self, dim: (i32, i32)) {
        self.file.refresh(dim)
    }

    split_func!(move_cursor_left -> bool);
    split_func!(move_cursor_right -> bool);
    restrict_func!(move_cursor_up -> bool, false);
    restrict_func!(move_cursor_down -> bool, false);
    split_func!(move_cursor_home);
    split_func!(move_cursor_end);
    restrict_func!(page_up);
    restrict_func!(page_down);

    pub fn insert_newline(&mut self, dim: (i32, i32)) {
        match self.one_liner {
            Some(_) => panic!("Can't insert newline in one liner! That's the point!"),
            None => self.file.insert_newline(dim)
        }
    }

    split_func!(delete);
    split_func!(backspace);

    pub fn insert(&mut self, dim: (i32, i32), c: char) {
        match self.one_liner {
            Some(ref mut ols) => ols.file.insert(dim, c),
            None => self.file.insert(dim, c)
        }
    }

    pub fn save_file(&self, path: &str) {
        self.file.save(path);
    }
}
