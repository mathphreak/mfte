use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;

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
}
