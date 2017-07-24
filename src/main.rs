#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;

use std::io::Write;
use std::env;

mod keybinds;
use keybinds::*;

mod file;
use file::*;

mod terminal;
use terminal::*;

const LINENO_CHARS : i32 = 3;

fn render_file(mut out: &mut Terminal, file: &File) {
    let (screen_width, screen_height) = out.get_size();

    let mut x = 1;
    let mut y = 1;

    for (line_number_maybe, line) in file.wrapped_lines((screen_width - LINENO_CHARS - 1, screen_height)) {
        if let Some(line_number) = line_number_maybe {
            out.goto((x, y));
            out.set_color_fg(Color::Grey);
            write!(out, "{:1$}",
                   line_number + 1, LINENO_CHARS as usize).unwrap();
            out.set_color_fg(Color::Reset);
        }
        x += LINENO_CHARS + 1;
        out.goto((x, y));
        write!(out, "{}", line).unwrap();
        x = 1;
        y += 1;
    }
}

fn render_footer(mut out: &mut Terminal, keys: &KeybindTable) {
    let (screen_width, screen_height) = out.get_size();

    let mut x = 1;
    let mut y = screen_height - 1;
    for (key, action) in keys.entries() {
        out.goto((x, y));
        out.set_color_fg(Color::Black);
        out.set_color_bg(Color::White);
        write!(out, "{}", key).unwrap();
        out.set_color_fg(Color::Reset);
        out.set_color_bg(Color::Reset);
        write!(out, " {}", action).unwrap();
        x += 16;
        if x > screen_width {
            x = 1;
            y -= 1;
        }
    }
}

fn main() {
    let keys = KeybindTable::default();
    let mut term = Terminal::default();
    term.clear();
    render_footer(&mut term, &keys);
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned().unwrap_or(String::from("README.md"));
    let file = File::open(&filename);
    render_file(&mut term, &file);
    let (mut cursor_x, mut cursor_y) = (1, 1);
    term.goto((cursor_x + LINENO_CHARS + 1, cursor_y));
    term.flush().unwrap();
    for evt in term.keys() {
        match evt {
            Event::Key(Key::Ctrl(k)) => {
                match keys.lookup(Key::Ctrl(k)) {
                    Some(Command::Quit) => break,
                    _ => ()
                }
            },
            Event::Key(Key::Left) => cursor_x -= 1,
            Event::Key(Key::Right) => cursor_x += 1,
            Event::Key(Key::Up) => cursor_y -= 1,
            Event::Key(Key::Down) => cursor_y += 1,
            _ => {}
        }
        let (screen_width, screen_height) = term.get_size();
        if cursor_x < 1 { cursor_x = 1; }
        if cursor_x > screen_width - LINENO_CHARS - 1 { cursor_x = screen_width - LINENO_CHARS - 2; }
        if cursor_y < 1 { cursor_y = 1; }
        if cursor_y > screen_height - 3 { cursor_y = screen_height - 3; }
        term.goto((cursor_x + LINENO_CHARS + 1, cursor_y));
        term.flush().unwrap();
    }
}
