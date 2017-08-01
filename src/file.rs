use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::cmp;

pub struct Cursor {
    pub x: i32,
    pub y: i32,
}

pub struct File {
    pub name: String,
    pub lines: Vec<String>,
    file_cursor: Cursor,
    y_offset: i32,
    last_dim: (i32, i32),
    misc: String,
}

impl File {
    pub fn debug(&self, dim: (i32, i32)) -> String {
        format!("FC ({}, {}) O ({}) C ({}, {}) {}",
            self.file_cursor.x, self.file_cursor.y, self.y_offset,
            self.cursor(dim).x, self.cursor(dim).y, self.misc
        )
    }

    pub fn open(path: &str) -> File {
        let f = fs::File::open(path).expect("Could not open file");
        let f = io::BufReader::new(f);

        File {
            name: String::from(Path::new(path).file_name().unwrap().to_string_lossy()),
            lines: f.lines().map(|r| r.unwrap()).collect(),
            file_cursor: Cursor { x: 1, y: 1 },
            y_offset: 0,
            last_dim: (0, 0),
            misc: String::from(""),
        }
    }

    pub fn cursor(&self, dim: (i32, i32)) -> Cursor {
        Cursor {
            x: (self.file_cursor.x - 1) % dim.0 + 1,
            y: self.file_cursor.y + (self.file_cursor.x - 1) / dim.0 + self.y_offset,
        }
    }

    pub fn wrapped_lines(&self, dim: (i32, i32)) -> Vec<(Option<u16>, String)> {
        let mut result = vec![];
        let (width, _) = dim;
        for (line_number, raw_line) in self.lines.iter().enumerate() {
            let mut line_start = 0;
            let mut line_end = cmp::min(raw_line.len(), width as usize);
            result.push((Some(line_number as u16),
                         String::from(&raw_line[line_start..line_end])));
            while line_end < raw_line.len() {
                line_start = line_end;
                line_end += cmp::min(raw_line.len() - line_end, width as usize);
                result.push((None,
                             String::from(&raw_line[line_start..line_end])));
            }
        }
        result
    }

    fn current_line(&self) -> &String {
        &self.lines[self.file_cursor.y as usize - 1]
    }

    fn recompute_offsets(&mut self, dim: (i32, i32)) {
        if dim != self.last_dim {
            panic!("AAA I CAN'T DO THIS YET");
        }
    }

    pub fn move_cursor_left(&mut self, dim: (i32, i32)) {
        if self.file_cursor.x > 1 {
            self.file_cursor.x -= 1;
        } else if self.file_cursor.y > 1 {
            self.move_cursor_up(dim);
            self.file_cursor.x = self.current_line().len() as i32 + 1;
        }
    }

    pub fn move_cursor_right(&mut self, dim: (i32, i32)) {
        if self.file_cursor.x <= self.current_line().len() as i32 {
            self.file_cursor.x += 1;
        } else if self.file_cursor.y < self.lines.len() as i32 {
            // Depend on truncation here to clamp to lower multiple of screen width
            self.file_cursor.x = ((self.file_cursor.x - 1) / dim.0) * dim.0 + 1;
            self.move_cursor_down(dim);
        }
    }

    pub fn move_cursor_up(&mut self, dim: (i32, i32)) {
        if self.file_cursor.x > dim.0 {
            self.file_cursor.x -= dim.0;
        } else if self.file_cursor.y > 1 {
            self.file_cursor.y -= 1;
            let extra_lines = self.current_line().len() as i32 / dim.0;
            self.y_offset -= self.current_line().len() as i32 / dim.0;
            self.file_cursor.x += extra_lines * dim.0;
            if self.file_cursor.x > self.current_line().len() as i32 + 1 {
                self.file_cursor.x = self.current_line().len() as i32 + 1;
            }
        }
    }

    pub fn move_cursor_down(&mut self, dim: (i32, i32)) {
        if self.file_cursor.x / dim.0 < self.current_line().len() as i32 / dim.0 {
            self.file_cursor.x += dim.0;
        } else if self.file_cursor.y < self.lines.len() as i32 {
            self.file_cursor.y += 1;
            while self.file_cursor.x > dim.0 {
                self.file_cursor.x -= dim.0;
                self.y_offset += 1;
            }
        }
        if self.file_cursor.x > self.current_line().len() as i32 + 1 {
            self.file_cursor.x = self.current_line().len() as i32 + 1;
        }
    }

    pub fn insert(&mut self, dim: (i32, i32), c: char) {
        {
            let x = self.file_cursor.x - 1;
            let line = &mut self.lines[self.file_cursor.y as usize - 1];
            let pos = if x > line.len() as i32 {
                self.file_cursor.x = line.len() as i32 - 1;
                line.len()
            } else {
                x as usize
            };
            line.insert(pos, c);
        }
        self.move_cursor_right(dim);
    }
}
