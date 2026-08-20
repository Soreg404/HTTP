use std::ops::Range;

macro_rules! trace {
    ($($arg:tt)*) => {
        println!("\x1b[96mtrace!\x1b[0m [{}:{:04}] {}",
            file!(),
            line!(),
            format_args!($($arg)*));
    };
}

pub mod defs;
mod cadv;
#[expect(dead_code)]
mod first_line;
mod headers;

pub struct Collector {
    proc_bytes: usize,
    first_line_lf_idx: usize,
    last_header_lf_idx: usize,

    stage: Stage,
    advance_buffer: [u8; 0xff],
    advance_buffer_head: usize,

    content_length: Option<usize>,
}

#[derive(Debug, Copy, Clone)]
enum Stage {
    FirstLine,
    Headers(StageHeaders),
    BodyNormal(usize),
    #[expect(dead_code)]
    BodyChunkLength,
    BodyChunk(usize),
    Finished(Result<(), ()>)
}

#[derive(Debug, Copy, Clone)]
enum StageHeaders {
    HeaderName,
    HeaderValue,
    ContentLengthHeaderValue,
    TransferEncodingHeaderValue,
}

#[derive(Debug)]
pub enum AvAction {
    Nop,
    FirstLineReady(Range<usize>),
    HeadersReady(Range<usize>),
    BodyReady(Range<usize>),
    Finished(Result<(), ()>)
}

#[derive(Debug)]
pub struct Advance {
    pub current: usize,
    pub total: usize,
    pub av_action: AvAction
}

impl Collector {
    pub fn new() -> Self {
        Self {
            proc_bytes: 0,
            first_line_lf_idx: 0,
            last_header_lf_idx: 0,

            stage: Stage::FirstLine,
            advance_buffer: [0u8; 0xff],
            advance_buffer_head: 0,

            content_length: None,
        }
    }
    pub fn advance(&mut self, buffer: &[u8]) -> Advance {
        self.advance_inner(buffer)
    }

    pub fn finish_status(&self) -> Option<Result<(), ()>> {
        match self.stage {
            Stage::Finished(r) => Some(r),
            _ => None
        }
    }
    pub fn get_first_line_range(&self) -> Option<Range<usize>> {
        match self.stage {
            Stage::FirstLine => None,
            _ => Some(0..self.first_line_lf_idx + 1)
        }
    }
    pub fn get_headers_range(&self) -> Option<Range<usize>> {
        match self.stage {
            Stage::FirstLine | Stage::Headers(_) => None,
            _ => Some(self.first_line_lf_idx + 1..self.first_line_lf_idx + 1)
        }
    }
}

fn split_crlf(s: &[u8]) -> (&[u8], &[u8]) {
    let mut sp = s.len();
    match s.last() {
        Some(b'\n') => { sp -= 1; }
        _ => return (s, &[])
    }
    match &s[..sp].last() {
        Some(b'\r') => { sp -= 1; }
        _ => {}
    }
    s.split_at(sp)
}
#[test]
fn test_split_crlf() {
    assert_eq!(split_crlf(b"hello\r\n"), (b"hello".as_slice(), b"\r\n".as_slice()));
    assert_eq!(split_crlf(b"hello\n\r"), (b"hello\n\r".as_slice(), b"".as_slice()));
    assert_eq!(split_crlf(b"hello\n"),   (b"hello".as_slice(), b"\n".as_slice()));
}
