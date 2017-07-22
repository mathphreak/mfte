use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::cmp;

pub struct File {
    pub name: String,
    pub lines: Vec<String>
}

impl File {
    pub fn open(path: &str) -> File {
        let f = fs::File::open(path).expect("Could not open file");
        let f = io::BufReader::new(f);

        File {
            name: String::from(Path::new(path).file_name().unwrap().to_string_lossy()),
            lines: f.lines().map(|r| r.unwrap()).collect()
        }
    }

    pub fn wrapped_lines(&self, dim: (u16, u16)) -> Vec<(Option<u16>, String)> {
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
}
