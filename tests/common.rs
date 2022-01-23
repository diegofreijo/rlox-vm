use std::io::Write;

#[derive(Debug)]
pub struct Output {
    pub contents: String,
}

impl Output {
    pub fn new() -> Self {
        Output {
            contents: String::new(),
        }
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.contents.push_str(&std::str::from_utf8(buf).unwrap());

        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
