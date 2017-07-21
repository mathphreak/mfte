extern crate termion;

fn main() {
    print!("{}{}Stuff\n", termion::clear::All, termion::cursor::Goto(1, 1));
}
