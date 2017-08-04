use std::fs;
use std::io;
use std::io::prelude::*;
use std::cmp;
use std::fmt;

use super::terminal::Color;
use super::indent::Indented;

pub struct TextChunk {
    pub contents: String,
    pub background: Color,
    pub foreground: Color,
}

#[derive(Clone)]
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

impl PartialEq for Cursor {
    fn eq(&self, other: &Cursor) -> bool {
        self.y == other.y && self.x == other.x
    }
}

impl Eq for Cursor {
}

impl PartialOrd for Cursor {
    fn partial_cmp(&self, other: &Cursor) -> Option<cmp::Ordering> {
        if self.y < other.y {
            Some(cmp::Ordering::Less)
        } else if self.y > other.y {
            Some(cmp::Ordering::Greater)
        } else {
            self.x.partial_cmp(&other.x)
        }
    }
}

impl Ord for Cursor {
    fn cmp(&self, other: &Cursor) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
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
            self.move_down(dim, lines);
            self.x = 1;
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

    fn move_home(&mut self, dim: (i32, i32), lines: &Vec<String>) {
        if self.x <= dim.0 {
            // TODO make this not hard coded
            if let Some(s) = lines[self.y as usize - 1].indent_end(4) {
                if self.x != s as i32 + 1 {
                    self.x = s as i32 + 1;
                } else {
                    self.x = 1;
                }
            } else {
                self.x = 1;
            }
        } else {
            // Depend on truncation here to clamp to lower multiple of screen width
            self.x = ((self.x - 1) / dim.0) * dim.0 + 1;
        }
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
    selection_start: Option<Cursor>,
    selecting: bool,
    window_top: Cursor,
    last_dim: (i32, i32),
    tab_width: u8,
    misc: String,
}

impl File {
    pub fn debug(&self, dim: (i32, i32)) -> String {
        let selection_text = if let Some(ref c) = self.selection_start {
            format!("Selection {} ", c)
        } else {
            String::from("")
        };
        format!("{}Caret {}, Top {}, Cursor {} {}", selection_text,
            self.caret, self.window_top, self.cursor(dim), self.misc
        )
    }

    pub fn empty() -> File {
        File {
            name: String::from("<empty>"),
            lines: vec![String::from("")],
            caret: Cursor { x: 1, y: 1, y_offset: 0 },
            selection_start: None,
            selecting: false,
            window_top: Cursor { x: 1, y: 1, y_offset: 0 },
            last_dim: (0, 0),
            tab_width: 4,
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
            name: String::from(path),
            lines: lines,
            caret: Cursor { x: 1, y: 1, y_offset: 0 },
            selection_start: None,
            selecting: false,
            window_top: Cursor { x: 1, y: 1, y_offset: 0 },
            last_dim: (0, 0),
            tab_width: 4,
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
    
    fn chunk(&self, line_number: usize, mut line: String, offset: usize) -> Vec<TextChunk> {
        let mut result = vec![];
        let mut last = None;
        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        if let Some(ref sel) = self.selection_start {
            let start = cmp::min(sel, &self.caret);
            let end = cmp::max(sel, &self.caret);
            let sx = start.x as usize - 1;
            let ex = end.x as usize - 1;
            let sy = start.y as usize - 1;
            let ey = end.y as usize - 1;
            if line_number >= sy && line_number <= ey {
                if line_number == ey && offset + line.len() >= ex {
                    last = Some(TextChunk {
                        contents: line.split_off(ex - offset),
                        foreground: Color::Reset,
                        background: Color::Reset,
                    });
                } else {
                    // Throw in a space at the end to indicate that the selection includes the newline
                    line.push(' ');
                }
                if line_number == sy && offset < sx {
                    let real_line = line.split_off(sx - offset);
                    result.push(TextChunk {
                        contents: line,
                        foreground: Color::Reset,
                        background: Color::Reset,
                    });
                    line = real_line;
                }
                fg = Color::Black;
                bg = Color::White;
            }
        }
        result.push(TextChunk {
            contents: line,
            foreground: fg,
            background: bg,
        });
        if let Some(c) = last {
            result.push(c);
        }
        result
    }

    pub fn chunked_text(&self, dim: (i32, i32)) -> Vec<(Option<u16>, Vec<TextChunk>)> {
        let mut result = vec![];
        let width = dim.0 as usize;
        let top_y = self.window_top.y as usize - 1;
        let top_extra = (self.window_top.x - 1) / dim.0;
        for (line_number, raw_line) in self.lines.iter().enumerate().skip(top_y) {
            let mut line_start = 0;
            let mut line_end = cmp::min(raw_line.len(), width);
            let line = String::from(&raw_line[line_start..line_end]);
            let chunks = self.chunk(line_number, line, line_start);
            result.push((Some(line_number as u16), chunks));
            while line_end < raw_line.len() {
                line_start = line_end;
                line_end += cmp::min(raw_line.len() - line_end, width);
                let line = String::from(&raw_line[line_start..line_end]);
                let chunks = self.chunk(line_number, line, line_start);
                result.push((None, chunks));
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
    
    pub fn select(&mut self) {
        self.selecting = true;
        if self.selection_start.is_none() {
            self.selection_start = Some(self.caret.clone());
        }
    }
    
    pub fn deselect(&mut self) {
        self.selecting = false;
        self.selection_start = None;
    }
    
    fn tweak_selection(&mut self) {
        if self.selecting {
            self.selecting = false;
        } else {
            self.deselect();
        }
    }

    pub fn move_cursor_left(&mut self, dim: (i32, i32)) -> bool {
        self.tweak_selection();
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
        self.tweak_selection();
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
        self.tweak_selection();
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
        self.tweak_selection();
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
        self.tweak_selection();
        self.recompute_offsets(dim);
        self.caret.move_home(dim, &self.lines);
    }

    pub fn move_cursor_end(&mut self, dim: (i32, i32)) {
        self.tweak_selection();
        self.recompute_offsets(dim);
        self.caret.move_end(dim, &self.lines);
    }

    pub fn page_up(&mut self, dim: (i32, i32)) {
        self.tweak_selection();
        for _ in 0..dim.1 {
            self.move_cursor_up(dim);
        }
    }

    pub fn page_down(&mut self, dim: (i32, i32)) {
        self.tweak_selection();
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
            let end = self.current_line().indent_end(self.tab_width);
            let line = &mut self.lines[self.caret.y as usize - 1];
            let mut indented = end.is_some();
            if let Some(s) = end {
                indented = x as i32 <= s - self.tab_width as i32;
            }
            if indented {
                line.pop_indentation(self.tab_width);
            } else {
                line.remove(x);
            }
        }
    }

    pub fn backspace(&mut self, dim: (i32, i32)) {
        let x = self.caret.x as usize - 1;
        if x as i32 <= self.current_line().indent_end(self.tab_width).unwrap_or(-1) {
            self.caret.x -= self.tab_width as i32;
        } else {
            self.move_cursor_left(dim);
        }
        self.delete(dim);
    }

    pub fn insert_newline(&mut self, dim: (i32, i32)) {
        let (mut after, n) = {
            let before = &mut self.lines[self.caret.y as usize - 1];
            let n = before.indent_end(self.tab_width);
            (before.split_off(self.caret.x as usize - 1), n)
        };
        if let Some(n) = n {
            after.insert_str(0, &" ".repeat(n as usize));
        }
        self.lines.insert(self.caret.y as usize, after);
        self.move_cursor_right(dim);
        if let Some(n) = n {
            self.caret.x += n;
        }
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
