use std::fs;
use std::io;
use std::io::prelude::*;
use std::cmp;
use std::fmt;

use super::terminal::Color;
use super::indent::Indented;
use super::config::Config;

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

    fn move_home(&mut self, dim: (i32, i32), lines: &Vec<String>, indent_size: u8) {
        if self.x <= dim.0 {
            // TODO make this not hard coded
            if let Some(s) = lines[self.y as usize - 1].indent_end(indent_size) {
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
    pub caret: Cursor,
    selection_start: Option<Cursor>,
    selecting: bool,
    window_top: Cursor,
    last_dim: (i32, i32),
    misc: String,
    pub display_dirty: bool,
    contents_dirty: bool,
    config: Config,
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
            misc: String::from(""),
            display_dirty: false,
            contents_dirty: false,
            config: Config::config_for(None),
        }
    }

    pub fn open(path: &str) -> File {
        let mut lines: Vec<String> = match fs::File::open(path) {
            Ok(f) => {
                let f = io::BufReader::new(f);
                f.lines().map(|r| r.unwrap()).collect()
            },
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    vec![]
                } else {
                    panic!("Could not open file: {}", e);
                }
            }
        };
        if lines.len() == 0 {
            lines.push(String::from(""));
        }

        let config = Config::config_for(Some(path));
        let misc = if config.indent() == "\t" {
            "Tabs are stupid and MFTE doesn't support them."
        } else {
            ""
        };

        File {
            name: String::from(path),
            lines: lines,
            caret: Cursor { x: 1, y: 1, y_offset: 0 },
            selection_start: None,
            selecting: false,
            window_top: Cursor { x: 1, y: 1, y_offset: 0 },
            last_dim: (0, 0),
            misc: String::from(misc),
            display_dirty: false,
            contents_dirty: false,
            config: Config::config_for(Some(path)),
        }
    }

    pub fn save(&mut self, path: &str) {
        let f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .expect("Could not open file");
        let mut f = io::BufWriter::new(f);
        let ls = self.config.line_sep();

        if self.config.trim_trailing_whitespace {
            for line in self.lines.iter_mut() {
                *line = line.trim_right().to_string();
            }
        }

        let mut it = self.lines.iter().peekable();

        while it.peek().is_some() {
            let line = it.next().unwrap();
            write!(f, "{}", line).unwrap();
            if it.peek().is_some() || self.config.insert_final_newline {
                write!(f, "{}", ls).unwrap();
            }
        }

        self.contents_dirty = false;
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

    pub fn label(&self) -> String {
        let mut result = if self.contents_dirty {
            String::from("*")
        } else {
            String::from("")
        };
        result.push_str(&self.name);
        result
    }

    fn chunk(&self, line_number: usize, mut line: String, offset: usize, partial: bool) -> Vec<TextChunk> {
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
                        contents: line.split_off(ex - cmp::min(ex, offset)),
                        foreground: Color::Reset,
                        background: Color::Reset,
                    });
                } else if !partial {
                    // Throw in a space at the end to indicate that the selection includes the newline
                    line.push(' ');
                }
                if line_number == sy && offset < sx {
                    let line_length = line.len();
                    let real_line = line.split_off(cmp::min(line_length, sx - offset));
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
            let chunks = self.chunk(line_number, line, line_start, line_end < raw_line.len());
            result.push((Some(line_number as u16), chunks));
            while line_end < raw_line.len() {
                line_start = line_end;
                line_end += cmp::min(raw_line.len() - line_end, width);
                let line = String::from(&raw_line[line_start..line_end]);
                let chunks = self.chunk(line_number, line, line_start, line_end < raw_line.len());
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

    fn tab_width(&self) -> u8 {
        self.config.indent().len() as u8
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
        self.display_dirty = true;
    }

    pub fn select(&mut self) {
        self.selecting = true;
        if self.selection_start.is_none() {
            self.selection_start = Some(self.caret.clone());
            self.display_dirty = true;
        }
    }

    pub fn deselect(&mut self) {
        self.selecting = false;
        if self.selection_start.is_some() {
            self.selection_start = None;
            self.display_dirty = true;
        }
    }

    fn tweak_selection(&mut self) {
        if self.selecting {
            self.selecting = false;
            self.display_dirty = true;
        } else {
            self.deselect();
        }
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    pub fn selected_text(&self) -> String {
        if let Some(ref sel) = self.selection_start {
            let start = cmp::min(sel, &self.caret);
            let end = cmp::max(sel, &self.caret);
            let mut pos = (*start).clone();
            let mut here = self.lines[pos.y as usize - 1].clone();
            let mut result = here.split_off(pos.x as usize - 1);
            pos.x = 1;
            pos.y += 1;
            while pos.y <= end.y {
                result.push('\n');
                result.push_str(&self.lines[pos.y as usize - 1]);
                pos.y += 1;
            }
            let rl = result.len();
            result.split_off(rl - (self.lines[pos.y as usize - 2].len() - (end.x as usize - 1)));
            result
        } else {
            String::from("")
        }
    }

    pub fn move_cursor_left(&mut self, dim: (i32, i32)) {
        self.tweak_selection();
        self.recompute_offsets(dim);
        self.caret.move_left(dim, &self.lines);
        if self.cursor(dim).y < 1 {
            self.window_top.move_up(dim, &self.lines);
            self.display_dirty = true;
        }
    }

    pub fn move_cursor_right(&mut self, dim: (i32, i32)) {
        self.tweak_selection();
        self.recompute_offsets(dim);
        self.caret.move_right(dim, &self.lines);
        if self.cursor(dim).y > dim.1 {
            self.window_top.move_down(dim, &self.lines);
            self.display_dirty = true;
        }
    }

    pub fn move_cursor_up(&mut self, dim: (i32, i32)) {
        self.tweak_selection();
        self.recompute_offsets(dim);
        self.caret.move_up(dim, &self.lines);
        if self.cursor(dim).y < 1 {
            self.window_top.move_up(dim, &self.lines);
            self.display_dirty = true;
        }
    }

    pub fn move_cursor_down(&mut self, dim: (i32, i32)) {
        self.tweak_selection();
        self.recompute_offsets(dim);
        self.caret.move_down(dim, &self.lines);
        if self.cursor(dim).y > dim.1 {
            self.window_top.move_down(dim, &self.lines);
            self.display_dirty = true;
        }
    }

    pub fn move_cursor_home(&mut self, dim: (i32, i32)) {
        self.tweak_selection();
        self.recompute_offsets(dim);
        let w = self.tab_width();
        self.caret.move_home(dim, &self.lines, w);
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

    pub fn goto(&mut self, dim: (i32, i32), target: (i32, i32)) {
        let (row, col) = target;
        while self.caret.y < row {
            self.move_cursor_down(dim);
        }
        while self.caret.y > row {
            self.move_cursor_up(dim);
        }
        self.caret.x = if col < 1 {
            1
        } else if col > self.current_line().len() as i32 + 1 {
            self.current_line().len() as i32 + 1
        } else {
            col
        }
    }

    pub fn scroll_up(&mut self, dim: (i32, i32)) {
        for _ in 0..3 {
            self.window_top.move_up(dim, &self.lines);
        }
        self.display_dirty = true;
    }

    pub fn scroll_down(&mut self, dim: (i32, i32)) {
        for _ in 0..3 {
            self.window_top.move_down(dim, &self.lines);
        }
        self.display_dirty = true;
    }

    fn delete_selection(&mut self, dim: (i32, i32)) {
        if let Some(sel) = self.selection_start.take() {
            // This is not smart. Especially if the selection includes half of an indent level.
            let start = cmp::min(sel.clone(), self.caret.clone());
            self.caret = cmp::max(sel, self.caret.clone());
            while self.caret > start {
                self.backspace(dim);
            }
        }
    }

    pub fn insert(&mut self, dim: (i32, i32), c: char) {
        self.delete_selection(dim);
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
        self.display_dirty = true;
        self.contents_dirty = true;
        self.move_cursor_right(dim);
    }

    pub fn delete(&mut self, dim: (i32, i32)) {
        let x = self.caret.x as usize - 1;
        if self.selection_start.is_some() {
            self.delete_selection(dim);
        } else if x == self.current_line().len() {
            let y = self.caret.y as usize - 1;
            if y < self.lines.len() - 1 {
                let next_line = self.lines.remove(y + 1);
                self.lines[y].push_str(&next_line);
            }
        } else {
            let w = self.tab_width();
            let end = self.current_line().indent_end(w);
            let line = &mut self.lines[self.caret.y as usize - 1];
            let mut indented = end.is_some();
            if let Some(s) = end {
                indented = x as i32 <= s - w as i32;
            }
            if indented {
                line.pop_indentation(w);
            } else {
                line.remove(x);
            }
        }
        self.display_dirty = true;
        self.contents_dirty = true;
    }

    pub fn backspace(&mut self, dim: (i32, i32)) {
        let x = self.caret.x - 1;
        if self.selection_start.is_some() {
            self.delete_selection(dim);
            return;
        }
        if x - self.tab_width() as i32 >= 0 && x <= self.current_line().indent_end(self.tab_width()).unwrap_or(-1) {
            self.caret.x -= self.tab_width() as i32;
        } else {
            self.move_cursor_left(dim);
        }
        self.delete(dim);
    }

    pub fn tab(&mut self, dim: (i32, i32)) {
        for _ in 0..(self.tab_width()) {
            self.insert(dim, ' ')
        }
    }

    pub fn insert_newline(&mut self, dim: (i32, i32), indent: bool) {
        self.delete_selection(dim);
        let w = self.tab_width();
        let (mut after, mut n) = {
            let before = &mut self.lines[self.caret.y as usize - 1];
            let n = before.indent_end(w);
            (before.split_off(self.caret.x as usize - 1), n)
        };
        if !indent {
            n = None;
        }
        if let Some(n) = n {
            after.insert_str(0, &" ".repeat(n as usize));
        }
        self.lines.insert(self.caret.y as usize, after);
        self.move_cursor_right(dim);
        if let Some(n) = n {
            self.caret.x += n;
        }
        self.display_dirty = true;
        self.contents_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_save_preserves_everything() {
        let mut f = File::open("README.md");
        f.save("readme.bak");
        let orig = fs::File::open("README.md").unwrap();
        let new = fs::File::open("readme.bak").unwrap();
        for (b1, b2) in orig.bytes().zip(new.bytes()) {
            assert_eq!(b1.unwrap(), b2.unwrap());
        }
        fs::remove_file("readme.bak").unwrap();
    }

    #[test]
    fn selection_on_wrapped_line_going_forward() {
        let mut f = File::open("README.md");
        f.select();
        f.move_cursor_right((10, 10));
        f.chunked_text((10, 10));
    }

    #[test]
    fn selection_on_wrapped_line_going_backward() {
        let mut f = File::open("README.md");
        f.move_cursor_right((10, 10));
        f.select();
        f.move_cursor_left((10, 10));
        f.chunked_text((10, 10));
    }

    #[test]
    fn selection_on_wrapped_line_going_backward_from_end_of_line() {
        let mut f = File::open("README.md");
        for _ in 0..f.current_line().len() {
            f.move_cursor_right((10, 10));
        }
        f.select();
        f.move_cursor_left((10, 10));
        f.chunked_text((10, 10));
    }
}
