use std::collections::HashMap;
use super::terminal::Key;

const DEFAULT_KEYBINDS: &'static str = r#"^Q: Quit
^W: CloseFile
^O: OpenFile
^S: SaveFile
^X: Cut
^C: Copy
^V: Paste
^F: Find
^H: FindReplace
^Z: Undo
^Y: Redo
^R: Refresh
^T: NewTab
^G: Goto
"#;

macro_attr! {
    #[derive(Clone, EnumDisplay!, EnumFromStr!)]
    pub enum Command {
        Quit,
        CloseFile,
        OpenFile,
        SaveFile,
        Cut,
        Copy,
        Paste,
        Find,
        FindReplace,
        Undo,
        Redo,
        Refresh,
        NewTab,
        Goto,
    }
}

pub struct KeybindTable {
    table: HashMap<Key, Command>
}

impl KeybindTable {
    pub fn lookup(&self, key: Key) -> Option<Command> {
        self.table.get(&key).cloned()
    }

    pub fn entries(&self) -> Vec<(String, String)> {
        self.table.iter().map(|(key, command)| {
            let key = match key {
                &Key::Ctrl(ref k) => match **k {
                    Key::Char(ref k) => format!("^{}", *k).to_uppercase(),
                    _ => String::from("???")
                },
                _ => String::from("???")
            };
            (key, format!("{}", command))
        }).collect()
    }
}

impl Default for KeybindTable {
    fn default() -> Self {
        Self::from(DEFAULT_KEYBINDS)
    }
}

fn decode_key_spec(spec: &str) -> Option<Key> {
    if spec.len() != 2 {
        println!("Bad key specifier: {}", spec);
        return None;
    }
    let (modifier, key) = spec.split_at(1);
    let modifier = modifier.chars().next().unwrap();
    let key = key.chars().next().unwrap();
    if modifier != '^' {
        println!("Bad key specifier: {}", spec);
        return None;
    }
    Some(Key::Ctrl(Box::new(Key::Char(key))))
}

impl<'a> From<&'a str> for KeybindTable {
    fn from(text: &'a str) -> Self {
        let mut result = KeybindTable {
            table: HashMap::new()
        };
        for line in text.lines() {
            let data: Vec<_> = line.split(": ").collect();
            if data.len() != 2 {
                println!("Bad keybind specifier: {}", line);
                continue
            }
            if let Ok(command) = data[1].parse::<Command>() {
                let key_spec = data[0].to_lowercase();
                match decode_key_spec(&key_spec) {
                    Some(key) => {
                        result.table.insert(key, command);
                    },
                    _ => {}
                }
            } else {
                println!("Bad command: {}", data[1]);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn ctrl(c: char) -> Key {
        Key::Ctrl(Box::new(Key::Char(c)))
    }
    
    #[test]
    fn keybind_parsing_works() {
        let keys = KeybindTable::from("^Q: Quit");
        match keys.lookup(ctrl('q')) {
            Some(Command::Quit) => (),
            _ => panic!("Looking up ^Q failed!")
        }
        match keys.lookup(ctrl('x')) {
            None => (),
            _ => panic!("Looking up ^X succeeded!")
        }
        let keys = KeybindTable::from("^X: Quit");
        match keys.lookup(ctrl('x')) {
            Some(Command::Quit) => (),
            _ => panic!("Looking up ^X failed!")
        }
        match keys.lookup(ctrl('q')) {
            None => (),
            _ => panic!("Looking up ^Q succeeded!")
        }
    }
}
