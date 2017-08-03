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

mod state;
use state::*;

mod indent;

fn get_file_size(term: &Terminal, state: &EditorState) -> (i32, i32) {
    let (screen_w, screen_h) = term.get_size();
    let one_liner_offset = match state.one_liner_active() {
        true => 1,
        false => 0
    };
    let tab_bar_offset = if state.files.len() > 1 {
        1
    } else {
        0
    };
    (screen_w - state.lineno_chars() - 1, screen_h - one_liner_offset - tab_bar_offset - 3)
}

fn render_file(out: &mut Terminal, state: &EditorState) {
    let file_size = get_file_size(out, state);
    let mut x = 1;
    let mut y = 1;

    let file = state.active_file();

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

fn render_footer(out: &mut Terminal, state: &EditorState) {
    let (screen_width, screen_height) = out.get_size();

    let mut x = 1;
    let mut y = screen_height - 1;
    for (key, action) in state.keys.entries() {
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

fn render_status(out: &mut Terminal, state: &EditorState) {
    let (_, height) = out.get_size();
    let file_size = get_file_size(out, state);
    let x = 1;
    let y = height;
    out.goto((x, y));
    write!(out, "{}", state.debug(file_size)).unwrap();
}

fn render_one_liner(out: &mut Terminal, state: &EditorState) {
    if let &Some(ref ols) = state.one_liner() {
        let (_, screen_height) = out.get_size();
        let mut y = screen_height - 3;
        if state.files.len() > 1 {
            y -= 1;
        }

        out.goto((1, y));
        out.set_color_fg(Color::Black);
        out.set_color_bg(Color::White);
        write!(out, "{} ", ols.label).unwrap();
        out.set_color_fg(Color::Reset);
        out.set_color_bg(Color::Reset);
        write!(out, "{}", ols.value()).unwrap();
    }
}

fn render_tab_bar(out: &mut Terminal, state: &EditorState) {
    if state.files.len() > 1 {
        let (_, screen_height) = out.get_size();
        let mut x = 1;
        let y = screen_height - 3;

        for (i, f) in state.files.iter().enumerate() {
            out.goto((x, y));
            out.set_color_fg(Color::Black);
            if i == state.active_file {
                out.set_color_bg(Color::White);
            } else {
                out.set_color_bg(Color::Grey);
            }
            write!(out, "{}", f.name).unwrap();
            x += f.name.len() as i32 + 1;
        }
        out.set_color_fg(Color::Reset);
        out.set_color_bg(Color::Reset);
    }
}

fn main() {
    let mut term = Terminal::default();
    term.clear();
    term.flush().unwrap();
    let mut state = EditorState {
        keys: KeybindTable::default(),
        files: vec![],
        one_liners: vec![],
        active_file: 0,
    };
    for filename in env::args().skip(1) {
        state.files.push(File::open(&filename));
        state.one_liners.push(None);
    }
    if state.files.len() == 0 {
        state.files.push(File::empty());
        state.one_liners.push(None);
    }
    render_footer(&mut term, &state);
    render_file(&mut term, &state);
    render_status(&mut term, &state);
    render_tab_bar(&mut term, &state);
    let file_size = get_file_size(&term, &state);
    term.goto(state.cursor(file_size));
    term.flush().unwrap();
    let mut file_lines = state.lines_len();
    let mut file_wrapped_lines = state.wrapped_lines(file_size).len();
    let mut file_dirty = false;
    let mut screen_dirty = false;
    for evt in term.keys() {
        let file_size = get_file_size(&term, &state);
        match evt {
            Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
                let left_gutter = state.lineno_chars() + 1;
                let bottom_gutter = file_size.1 + 1;
                if x > left_gutter && y < bottom_gutter {
                    state.move_cursor_to(file_size, (x, y));
                } else if y < bottom_gutter {
                    let x = state.cursor(file_size).0;
                    state.move_cursor_to(file_size, (x, y));
                } else if y == bottom_gutter && state.files.len() > 1 && !state.one_liner_active() {
                    let mut tab_x = 1;
                    for (i, f) in state.files.iter().enumerate() {
                        tab_x += f.name.len() as i32 + 1;
                        if x < tab_x {
                            state.active_file = i;
                            break;
                        }
                    }
                    screen_dirty = true;
                }
            },
            Event::Mouse(_) => (),
            Event::Unsupported(_) => (),
            Event::Key(Key::Null) => (),
            Event::Key(Key::Insert) => (),
            Event::Key(Key::F(_)) => (),
            Event::Key(Key::Esc) => {
                state.one_liner_mut().take();
                screen_dirty = true;
            },
            Event::Key(Key::Ctrl(ref k)) if **k == Key::Char('\t') => {
                state.next_tab();
                screen_dirty = true;
            }
            Event::Key(k @ Key::Ctrl(_)) | Event::Key(k @ Key::Alt(_)) => {
                match state.keys.lookup(k) {
                    Some(Command::Quit) => break,
                    Some(Command::Refresh) => {
                        state.refresh(file_size);
                        screen_dirty = true;
                    },
                    Some(Command::NewTab) => {
                        state.new_tab();
                        screen_dirty = true;
                    },
                    Some(Command::SaveFile) => {
                        let mut ols = OneLinerState::from(Command::SaveFile);
                        ols.file.lines[0] = state.active_file().name.clone();
                        ols.file.move_cursor_end(file_size);
                        state.set_one_liner(ols);
                        screen_dirty = true;
                    },
                    Some(Command::OpenFile) => {
                        let ols = OneLinerState::from(Command::OpenFile);
                        state.set_one_liner(ols);
                        screen_dirty = true;
                    },
                    Some(Command::CloseFile) => {
                        state.close_tab();
                        if state.files.len() == 0 {
                            break;
                        }
                        screen_dirty = true;
                    },
                    None => (),
                    Some(c) => {
                        let mut ols = OneLinerState::from(c);
                        ols.label = "Nope.";
                        ols.file.lines[0] = String::from("That doesn't work yet. Press Esc to move on");
                        ols.file.move_cursor_end(file_size);
                        state.set_one_liner(ols);
                        screen_dirty = true;
                    }
                }
            },
            Event::Key(Key::Left) => {
                screen_dirty = state.move_cursor_left(file_size);
            },
            Event::Key(Key::Right) => {
                screen_dirty = state.move_cursor_right(file_size);
            },
            Event::Key(Key::Up) => {
                screen_dirty = state.move_cursor_up(file_size);
            },
            Event::Key(Key::Down) => {
                screen_dirty = state.move_cursor_down(file_size);
            },
            Event::Key(Key::Home) => state.move_cursor_home(file_size),
            Event::Key(Key::End) => state.move_cursor_end(file_size),
            Event::Key(Key::PageUp) => {
                state.page_up(file_size);
                screen_dirty = true;
            },
            Event::Key(Key::PageDown) => {
                state.page_down(file_size);
                screen_dirty = true;
            },
            Event::Key(Key::Char('\t')) if state.one_liner_active() => {
                if let &mut Some(ref mut ols) = state.one_liner_mut() {
                    match ols.command {
                        Command::SaveFile | Command::OpenFile => ols.tab(),
                        _ => ()
                    }
                }
                file_dirty = true;
            },
            Event::Key(Key::Char('\t')) => {
                for _ in 0..4 {
                    state.insert(file_size, ' ');
                }
                file_dirty = true;
            },
            Event::Key(Key::Char('\n')) => {
                if let Some((command, value)) = state.consume_one_liner() {
                    match command {
                        Command::SaveFile => {
                            state.save_file(&value);
                        },
                        Command::OpenFile => {
                            state.open_file(&value);
                        },
                        _ => ()
                    };
                } else {
                    state.insert_newline(file_size);
                }
                screen_dirty = true;
            },
            Event::Key(Key::Delete) => {
                state.delete(file_size);
                screen_dirty = true;
            },
            Event::Key(Key::Backspace) => {
                state.backspace(file_size);
                screen_dirty = true;
            },
            Event::Key(Key::Shift(ref k)) if k.is_char() => {
                match **k {
                    Key::Char(c) => {
                        state.insert(file_size, c);
                        file_dirty = true;
                    },
                    _ => panic!("This was just the right thing!")
                }
            },
            Event::Key(Key::Char(c)) => {
                state.insert(file_size, c);
                file_dirty = true;
            },
            Event::Key(Key::Shift(_)) => ()
        }
        let file_size = get_file_size(&term, &state);
        if screen_dirty {
            file_dirty = true;
        }
        if file_dirty {
            let new_file_lines = state.lines_len();
            let new_file_wrapped_lines = state.wrapped_lines(file_size).len();
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
            render_footer(&mut term, &state);
            render_tab_bar(&mut term, &state);
            screen_dirty = false;
        }
        if file_dirty {
            render_file(&mut term, &state);
            render_one_liner(&mut term, &state);
            file_dirty = false;
        }
        render_status(&mut term, &state);
        term.goto(state.cursor(file_size));
        term.flush().unwrap();
    }
}
