pub trait Indented {
    fn indent_end(&self, indent_size: u8) -> Option<i32>;
    fn pop_indentation(&mut self, indent_size: u8);
}

impl Indented for String {
    fn indent_end(&self, indent_size: u8) -> Option<i32> {
        let mut leading_spaces = self.len() - self.trim_left().len();
        if leading_spaces % (indent_size as usize) > 0 {
            leading_spaces -= leading_spaces % (indent_size as usize);
        }
        if leading_spaces > 0 {
            Some(leading_spaces as i32)
        } else {
            None
        }
    }

    fn pop_indentation(&mut self, indent_size: u8) {
        if let Some(end) = self.indent_end(indent_size) {
            let end = end as usize;
            self.drain((end - indent_size as usize)..end);
        }
    }
}
