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

fn render_file(out: &mut Terminal, file_size: (i32, i32), file: &File) {
    let mut x = 1;
    let mut y = 1;

    for (line_number_maybe, line) in file.wrapped_lines(file_size) {
        if let Some(line_number) = line_number_maybe {
            out.goto((x, y));
            out.set_color_fg(Color::Grey);
            write!(out, "{:1$}",
                   line_number + 1, file.lineno_chars() as usize).unwrap();
            out.set_color_fg(Color::Reset);
        }
        x += file.lineno_chars() + 1;
        out.goto((x, y));
        write!(out, "{}", line).unwrap();
        x = 1;
        y += 1;
    }
}

fn render_footer(out: &mut Terminal, keys: &KeybindTable) {
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

fn render_status(out: &mut Terminal, file_size: (i32, i32), file: &File) {
    let (_, height) = out.get_size();
    let x = 1;
    let y = height;
    out.goto((x, y));
    write!(out, "{}", file.debug(file_size)).unwrap();
}

fn get_file_size(term: &Terminal, file: &File) -> (i32, i32) {
    let (screen_w, screen_h) = term.get_size();
    (screen_w - file.lineno_chars() - 1, screen_h - 3)
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
    let file_size = get_file_size(&term, &file);
    render_file(&mut term, file_size, &file);
    render_status(&mut term, file_size, &file);
    term.goto((file.cursor(file_size).x + file.lineno_chars() + 1, file.cursor(file_size).y));
    term.flush().unwrap();
    let mut file_lines = file.lines.len();
    let mut file_wrapped_lines = file.wrapped_lines(file_size).len();
    let mut file_dirty = false;
    let mut screen_dirty = false;
    for evt in term.keys() {
        let file_size = get_file_size(&term, &file);
        match evt {
            Event::Mouse(_) => (),
            Event::Unsupported(_) => (),
            Event::Key(Key::Null) => (),
            Event::Key(Key::Insert) => (),
            Event::Key(Key::F(_)) => (),
            Event::Key(Key::Esc) => (),
            Event::Key(k @ Key::Ctrl(_)) | Event::Key(k @ Key::Alt(_)) => {
                match keys.lookup(k) {
                    Some(Command::Quit) => break,
                    Some(Command::Refresh) => {
                        file.refresh(file_size);
                        screen_dirty = true;
                    },
                    None => (),
                    _ => ()
                }
            },
            Event::Key(Key::Left) => {
                screen_dirty = file.move_cursor_left(file_size);
            },
            Event::Key(Key::Right) => {
                screen_dirty = file.move_cursor_right(file_size);
            },
            Event::Key(Key::Up) => {
                screen_dirty = file.move_cursor_up(file_size);
            },
            Event::Key(Key::Down) => {
                screen_dirty = file.move_cursor_down(file_size);
            },
            Event::Key(Key::Home) => file.move_cursor_home(file_size),
            Event::Key(Key::End) => file.move_cursor_end(file_size),
            Event::Key(Key::PageUp) => {
                file.page_up(file_size);
                screen_dirty = true;
            },
            Event::Key(Key::PageDown) => {
                file.page_down(file_size);
                screen_dirty = true;
            },
            Event::Key(Key::Char('\t')) => {
                for _ in 0..4 {
                    file.insert(file_size, ' ');
                }
                file_dirty = true;
            },
            Event::Key(Key::Char('\n')) => {
                file.insert_newline(file_size);
                screen_dirty = true;
            },
            Event::Key(Key::Delete) => {
                file.delete(file_size);
                screen_dirty = true;
            },
            Event::Key(Key::Backspace) => {
                file.backspace(file_size);
                screen_dirty = true;
            },
            Event::Key(Key::Char(c)) => {
                file.insert(file_size, c);
                file_dirty = true;
            },
        }
        if screen_dirty {
            file_dirty = true;
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
            render_file(&mut term, file_size, &file);
            file_dirty = false;
        }
        render_status(&mut term, file_size, &file);
        term.goto((file.cursor(file_size).x + file.lineno_chars() + 1, file.cursor(file_size).y));
        term.flush().unwrap();
    }
}
