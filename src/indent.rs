pub trait Indented {
    fn indent_end(&self, indent_size: u8) -> Option<i32>;
    fn pop_indentation(&mut self, indent_size: u8);
}

impl Indented for String {
    fn indent_end(&self, indent_size: u8) -> Option<i32> {
        self.rfind(&" ".repeat(indent_size as usize)).map(|e| e as i32 + indent_size as i32)
    }

    fn pop_indentation(&mut self, indent_size: u8) {
        if let Some(end) = self.indent_end(indent_size) {
            let end = end as usize;
            self.drain((end - indent_size as usize)..end);
        }
    }
}
