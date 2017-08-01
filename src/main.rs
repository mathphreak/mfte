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

fn render_status(mut out: &mut Terminal, file: &File) {
    let (width, height) = out.get_size();
    let file_size = (width - LINENO_CHARS - 1, height - 3);
    let x = 1;
    let y = height;
    out.goto((x, y));
    write!(out, "{}", file.debug(file_size)).unwrap();
}

fn main() {
    let keys = KeybindTable::default();
    let mut term = Terminal::default();
    term.clear();
    term.flush().unwrap();
    render_footer(&mut term, &keys);
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned().unwrap_or(String::from("README.md"));
    let mut file = File::open(&filename);
    render_file(&mut term, &file);
    render_status(&mut term, &file);
    let (screen_w, screen_h) = term.get_size();
    let file_size = (screen_w - LINENO_CHARS - 1, screen_h - 3);
    term.goto((file.cursor(file_size).x + LINENO_CHARS + 1, file.cursor(file_size).y));
    term.flush().unwrap();
    let mut file_lines = file.lines.len();
    let mut file_wrapped_lines = file.wrapped_lines(file_size).len();
    let mut file_dirty = false;
    let mut screen_dirty = false;
    for evt in term.keys() {
        let (screen_w, screen_h) = term.get_size();
        let file_size = (screen_w - LINENO_CHARS - 1, screen_h - 3);
        match evt {
            Event::Key(Key::Ctrl(k)) => {
                match keys.lookup(Key::Ctrl(k)) {
                    Some(Command::Quit) => break,
                    _ => ()
                }
            },
            Event::Key(Key::Left) => file.move_cursor_left(file_size),
            Event::Key(Key::Right) => file.move_cursor_right(file_size),
            Event::Key(Key::Up) => file.move_cursor_up(file_size),
            Event::Key(Key::Down) => file.move_cursor_down(file_size),
            Event::Key(Key::Char('\n')) => {
                file.insert_newline(file_size);
                file_dirty = true;
                screen_dirty = true;
            },
            Event::Key(Key::Char(c)) => {
                file.insert(term.get_size(), c);
                file_dirty = true;
            },
            _ => {}
        }
        if file_dirty {
            let new_file_lines = file.lines.len();
            let new_file_wrapped_lines = file.wrapped_lines(file_size).len();
            if new_file_lines != file_lines {
                screen_dirty = true;
                file_lines = new_file_lines;
            }
            if new_file_wrapped_lines != file_wrapped_lines {
                screen_dirty = true;
                file_wrapped_lines = new_file_wrapped_lines;
            }
        }
        if screen_dirty {
            term.clear();
            render_footer(&mut term, &keys);
            screen_dirty = false;
        }
        if file_dirty {
            render_file(&mut term, &file);
            file_dirty = false;
        }
        render_status(&mut term, &file);
        term.goto((file.cursor(file_size).x + LINENO_CHARS + 1, file.cursor(file_size).y));
        term.flush().unwrap();
    }
}
