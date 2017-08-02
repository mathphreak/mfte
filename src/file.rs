use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::cmp;
use std::fmt;

pub struct Cursor {
    pub x: i32,
    pub y: i32,
    y_offset: i32,
}

impl fmt::Display for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, {}, o={})", self.x, self.y, self.y_offset)
    }
}

impl Cursor {
    fn curr_len(&self, lines: &Vec<String>) -> i32 {
        lines[self.y as usize - 1].len() as i32
    }

    fn move_left(&mut self, dim: (i32, i32), lines: &Vec<String>) {
        if self.x > 1 {
            self.x -= 1;
        } else if self.y > 1 {
            self.move_up(dim, lines);
            self.x = self.curr_len(lines) + 1;
        }
    }

    fn move_right(&mut self, dim: (i32, i32), lines: &Vec<String>) {
        if self.x <= self.curr_len(lines) {
            self.x += 1;
        } else if self.y < lines.len() as i32 {
            self.move_home(dim, lines);
            self.move_down(dim, lines);
        }
    }

    fn move_up(&mut self, dim: (i32, i32), lines: &Vec<String>) {
        if self.x > dim.0 {
            self.x -= dim.0;
        } else if self.y > 1 {
            self.y -= 1;
            let line_len = self.curr_len(lines);
            let extra_lines = line_len / dim.0;
            self.y_offset -= extra_lines;
            self.x += extra_lines * dim.0;
            if self.x > line_len + 1 {
                self.x = line_len + 1;
            }
        }
    }

    fn move_down(&mut self, dim: (i32, i32), lines: &Vec<String>) {
        if (self.x - 1) / dim.0 < self.curr_len(lines) / dim.0 {
            self.x += dim.0;
        } else if self.y < lines.len() as i32 {
            self.y += 1;
            while self.x > dim.0 {
                self.x -= dim.0;
                self.y_offset += 1;
            }
        }
        if self.x > self.curr_len(lines) + 1 {
            self.x = self.curr_len(lines) + 1;
        }
    }

    fn move_home(&mut self, dim: (i32, i32), _: &Vec<String>) {
        // Depend on truncation here to clamp to lower multiple of screen width
        self.x = ((self.x - 1) / dim.0) * dim.0 + 1;
    }

    fn move_end(&mut self, dim: (i32, i32), lines: &Vec<String>) {
        self.x = ((self.x - 1) / dim.0 + 1) * dim.0;
        if self.x > self.curr_len(lines) + 1 {
            self.x = self.curr_len(lines) + 1;
        }
    }

    fn project(&self, dim: (i32, i32)) -> Cursor {
        Cursor {
            x: (self.x - 1) % dim.0 + 1,
            y: self.y + (self.x - 1) / dim.0 + self.y_offset,
            y_offset: 0,
        }
    }

    fn recompute_offset(&mut self, dim: (i32, i32), lines: &Vec<String>) {
        self.y_offset = 0;
        for i in 0..(self.y - 1) {
            self.y_offset += lines[i as usize].len() as i32 / dim.0;
        }
    }
}

pub struct File {
    pub name: String,
    pub lines: Vec<String>,
    caret: Cursor,
    window_top: Cursor,
    last_dim: (i32, i32),
    misc: String,
}

impl File {
    pub fn debug(&self, dim: (i32, i32)) -> String {
        format!("Caret {}, Top {}, Cursor {} {}",
            self.caret, self.window_top, self.cursor(dim), self.misc
        )
    }

    pub fn empty() -> File {
        File {
            name: String::from(""),
            lines: vec![String::from("")],
            caret: Cursor { x: 1, y: 1, y_offset: 0 },
            window_top: Cursor { x: 1, y: 1, y_offset: 0 },
            last_dim: (0, 0),
            misc: String::from(""),
        }
    }

    pub fn open(path: &str) -> File {
        let f = fs::File::open(path).expect("Could not open file");
        let f = io::BufReader::new(f);

        let mut lines: Vec<String> = f.lines().map(|r| r.unwrap()).collect();
        if lines.len() == 0 {
            lines.push(String::from(""));
        }

        File {
            name: String::from(Path::new(path).file_name().unwrap().to_string_lossy()),
            lines: lines,
            caret: Cursor { x: 1, y: 1, y_offset: 0 },
            window_top: Cursor { x: 1, y: 1, y_offset: 0 },
            last_dim: (0, 0),
            misc: String::from(""),
        }
    }

    pub fn save(&self, path: &str) {
        let f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .expect("Could not open file");
        let mut f = io::BufWriter::new(f);

        for line in self.lines.iter() {
            write!(f, "{}\n", line).unwrap();
        }
    }

    pub fn cursor(&self, dim: (i32, i32)) -> Cursor {
        let projected_caret = self.caret.project(dim);
        let projected_top = self.window_top.project(dim);
        Cursor {
            x: projected_caret.x,
            y: projected_caret.y - projected_top.y + 1,
            y_offset: 0,
        }
    }

    pub fn lineno_chars(&self) -> i32 {
        format!("{}", self.lines.len()).len() as i32
    }

    pub fn wrapped_lines(&self, dim: (i32, i32)) -> Vec<(Option<u16>, String)> {
        let mut result = vec![];
        let width = dim.0 as usize;
        let top_y = self.window_top.y as usize - 1;
        let top_extra = (self.window_top.x - 1) / dim.0;
        for (line_number, raw_line) in self.lines.iter().enumerate().skip(top_y) {
            let mut line_start = 0;
            let mut line_end = cmp::min(raw_line.len(), width);
            result.push((Some(line_number as u16),
                         String::from(&raw_line[line_start..line_end])));
            while line_end < raw_line.len() {
                line_start = line_end;
                line_end += cmp::min(raw_line.len() - line_end, width);
                result.push((None,
                             String::from(&raw_line[line_start..line_end])));
            }
            if result.len() as i32 >= dim.1 + top_extra {
                break;
            }
        }
        result.truncate((dim.1 + top_extra) as usize);
        result.drain(..top_extra as usize);
        result
    }

    fn current_line(&self) -> &String {
        &self.lines[self.caret.y as usize - 1]
    }

    fn recompute_offsets(&mut self, dim: (i32, i32)) {
        if dim != self.last_dim {
            self.caret.recompute_offset(dim, &self.lines);
            self.window_top.recompute_offset(dim, &self.lines);
            self.last_dim = dim;
        }
    }

    pub fn refresh(&mut self, dim: (i32, i32)) {
        self.recompute_offsets(dim);
    }

    pub fn move_cursor_left(&mut self, dim: (i32, i32)) -> bool {
        self.recompute_offsets(dim);
        self.caret.move_left(dim, &self.lines);
        if self.cursor(dim).y < 1 {
            self.window_top.move_up(dim, &self.lines);
            true
        } else {
            false
        }
    }

    pub fn move_cursor_right(&mut self, dim: (i32, i32)) -> bool {
        self.recompute_offsets(dim);
        self.caret.move_right(dim, &self.lines);
        if self.cursor(dim).y > dim.1 {
            self.window_top.move_down(dim, &self.lines);
            true
        } else {
            false
        }
    }

    pub fn move_cursor_up(&mut self, dim: (i32, i32)) -> bool {
        self.recompute_offsets(dim);
        self.caret.move_up(dim, &self.lines);
        if self.cursor(dim).y < 1 {
            self.window_top.move_up(dim, &self.lines);
            true
        } else {
            false
        }
    }

    pub fn move_cursor_down(&mut self, dim: (i32, i32)) -> bool {
        self.recompute_offsets(dim);
        self.caret.move_down(dim, &self.lines);
        if self.cursor(dim).y > dim.1 {
            self.window_top.move_down(dim, &self.lines);
            true
        } else {
            false
        }
    }

    pub fn move_cursor_home(&mut self, dim: (i32, i32)) {
        self.recompute_offsets(dim);
        self.caret.move_home(dim, &self.lines);
    }

    pub fn move_cursor_end(&mut self, dim: (i32, i32)) {
        self.recompute_offsets(dim);
        self.caret.move_end(dim, &self.lines);
    }

    pub fn page_up(&mut self, dim: (i32, i32)) {
        for _ in 0..dim.1 {
            self.move_cursor_up(dim);
        }
    }

    pub fn page_down(&mut self, dim: (i32, i32)) {
        for _ in 0..dim.1 {
            self.move_cursor_down(dim);
        }
    }

    pub fn insert(&mut self, dim: (i32, i32), c: char) {
        {
            let x = self.caret.x - 1;
            let line = &mut self.lines[self.caret.y as usize - 1];
            let pos = if x > line.len() as i32 {
                self.caret.x = line.len() as i32 - 1;
                line.len()
            } else {
                x as usize
            };
            line.insert(pos, c);
        }
        self.move_cursor_right(dim);
    }

    pub fn delete(&mut self, _: (i32, i32)) {
        let x = self.caret.x as usize - 1;
        if x == self.current_line().len() {
            let y = self.caret.y as usize - 1;
            if y < self.lines.len() - 1 {
                let next_line = self.lines.remove(y + 1);
                self.lines[y].push_str(&next_line);
            }
        } else {
            let line = &mut self.lines[self.caret.y as usize - 1];
            line.remove(x);
        }
    }

    pub fn backspace(&mut self, dim: (i32, i32)) {
        self.move_cursor_left(dim);
        self.delete(dim);
    }

    pub fn insert_newline(&mut self, dim: (i32, i32)) {
        let after = {
            let before = &mut self.lines[self.caret.y as usize - 1];
            before.split_off(self.caret.x as usize - 1)
        };
        self.lines.insert(self.caret.y as usize, after);
        self.move_cursor_right(dim);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load_save_preserves_everything() {
        let f = File::open("README.md");
        f.save("readme.bak");
        let orig = fs::File::open("README.md").unwrap();
        let new = fs::File::open("readme.bak").unwrap();
        for (b1, b2) in orig.bytes().zip(new.bytes()) {
            assert_eq!(b1.unwrap(), b2.unwrap());
        }
        fs::remove_file("readme.bak").unwrap();
    }
}
