use super::*;

impl CHead {
    pub fn advance<'a>(&mut self, buffer: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        let mut i = 0;
        while i < buffer.len() {
            if buffer[i] == b'\n' {
                if self.crlf_flag {
                    return (buffer.split_at(i + 1));
                }
                self.crlf_flag = true;
            } else if buffer[i] != b'\r' {
                self.crlf_flag = false;
            }
            i += 1;
        }
        (buffer, &[])
    }

    pub fn is_ready(&self) {
        self.ready
    }
}
