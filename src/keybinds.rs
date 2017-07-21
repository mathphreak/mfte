use std::collections::HashMap;
use termion::event::Key;

macro_attr! {
    #[derive(Clone, EnumDisplay!, EnumFromStr!)]
    pub enum Command {
        Quit
    }
}

pub struct KeybindTable {
    table: HashMap<Key, Command>
}

impl KeybindTable {
    pub fn lookup(&self, key: Key) -> Option<Command> {
        self.table.get(&key).cloned()
    }
}

fn decode_key_spec(spec: &str) -> Option<Key> {
    if spec.len() != 2 {
        print!("Bad key specifier: {}", spec);
        return None;
    }
    let (modifier, key) = spec.split_at(1);
    let modifier = modifier.chars().next().unwrap();
    let key = key.chars().next().unwrap();
    if modifier != '^' {
        print!("Bad key specifier: {}", spec);
        return None;
    }
    Some(Key::Ctrl(key))
}

impl<'a> From<&'a str> for KeybindTable {
    fn from(text: &'a str) -> Self {
        let mut result = KeybindTable {
            table: HashMap::new()
        };
        for line in text.lines() {
            let data: Vec<_> = line.split(": ").collect();
            if data.len() != 2 {
                print!("Bad keybind specifier: {}", line);
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
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keybind_parsing_works() {
        let keys = KeybindTable::from("^Q: Quit");
        match keys.lookup(Key::Ctrl('q')) {
            Some(Command::Quit) => (),
            _ => panic!("Looking up ^Q failed!")
        }
        match keys.lookup(Key::Ctrl('x')) {
            None => (),
            _ => panic!("Looking up ^X succeeded!")
        }
        let keys = KeybindTable::from("^X: Quit");
        match keys.lookup(Key::Ctrl('x')) {
            Some(Command::Quit) => (),
            _ => panic!("Looking up ^X failed!")
        }
        match keys.lookup(Key::Ctrl('q')) {
            None => (),
            _ => panic!("Looking up ^Q succeeded!")
        }
    }
}
