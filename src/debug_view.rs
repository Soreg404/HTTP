use std::io::Write;
use crate::{Collector, Stage};

impl Collector {
    pub fn debug_view(&self, buffer: &[u8]) -> String {
        let mut s = Vec::new();
        self.debug_write(buffer, &mut s)
            .expect("debug write is fallible whatever");
        String::from_utf8_lossy(&s).to_string()
    }
    pub fn debug_write(&self, buffer: &[u8], sink: &mut impl Write) -> Result<(), std::io::Error> {
        writeln!(sink, "Collector:")?;
        write!(sink, "stage: ")?;
        match self.stage {
            Stage::FirstLine => writeln!(sink, "collecting first line")?,
            Stage::Headers(_) => writeln!(sink, "collecting headers")?,
            Stage::BodyNormal(len) => writeln!(sink, "collecting body (len={len})")?,
            Stage::BodyChunkLength => writeln!(sink, "collecting next chunk length")?, 
            Stage::BodyChunk(len) => writeln!(sink, "collecting chunk (len={len})")?,
            Stage::BodyChunkSkipCRLF => writeln!(sink, "collecting chunk - skipping trailing CRLF")?,
            Stage::Finished(r) => writeln!(sink, "finished {r:?}")?,
        }
        match self.get_first_line_range() {
            Some(r) => writeln!(
                sink, "first line:\n<<<{}>>>", String::from_utf8_lossy(&buffer[r]))?,
            None => writeln!(sink, "None")?,
        }
        match self.get_headers_range() {
            Some(r) => writeln!(
                sink, "headers:\n<<<{}>>>", String::from_utf8_lossy(&buffer[r]))?,
            None => writeln!(sink, "None")?,
        }
        match self.get_body_start_index() {
            Some(idx) => writeln!(
                sink, "body:\n<<<{}>>>",
                String::from_utf8_lossy(&buffer[idx..self.proc_bytes]))?,
            None => writeln!(sink, "None")?,
        }
        Ok(())
    }
}
