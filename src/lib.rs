#[macro_use]
mod helpers;

pub mod defs;
pub mod first_line;
pub mod headers;

mod cadv;

/// todo: change to new Ranges (since Rust 1.96.0)
use std::ops::Range;

mod debug_view;

pub struct Collector {
    proc_bytes: usize,
    first_line_lf_idx: usize,
    last_header_lf_idx: usize,

    stage: Stage,
    advance_buffer: [u8; 0xff],
    advance_buffer_head: usize,

    content_length: Option<usize>,

    chunk_start_idx: usize,
    transfer_encoding: Option<bool>,
}

#[derive(Debug, Copy, Clone)]
enum Stage {
    /// Collector collects first line the same way,
    /// regardless of which side it is.
    /// Distinction is made later, when decoding first line bytes.
    /// todo: maybe rename to MessageFirstLine
    FirstLine,
    Headers(StageHeaders),
    BodyNormal(usize),
    BodyChunkLength,
    BodyChunk(usize),
    BodyChunkSkipCRLF,
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
    BodyChunkReady(Range<usize>),
}

#[derive(Debug)]
pub struct Advance<'a> {
    pub processed: &'a [u8],
    pub rest: &'a [u8],
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

            chunk_start_idx: 0,
            transfer_encoding: None,
        }
    }
    pub fn advance<'a>(&mut self, buffer: &'a [u8]) -> Advance<'a> {
        self.advance_inner(buffer)
    }
    pub fn advance_ignore(&mut self, mut buffer: &[u8]) {
        while !buffer.is_empty() && !self.is_finished() {
            let advance = self.advance(buffer);
            buffer = advance.rest;
            match advance.av_action {
                AvAction::Nop => return,
                _ => {}
            }
        }
    }
    pub fn finish_status(&self) -> Option<Result<(), ()>> {
        match self.stage {
            Stage::Finished(r) => Some(r),
            _ => None
        }
    }
    pub fn is_finished(&self) -> bool {
        self.finish_status().is_some()
    }
    pub fn get_proc_bytes(&self) -> usize {
        self.proc_bytes
    }
    pub fn get_first_line_range(&self) -> Option<Range<usize>> {
        match self.stage {
            Stage::Finished(Err(())) => None,
            Stage::FirstLine => None,
            _ => Some(0 .. self.first_line_lf_idx + 1)
        }
    }
    pub fn get_headers_range(&self) -> Option<Range<usize>> {
        match self.stage {
            Stage::Finished(Err(())) => None,
            Stage::FirstLine | Stage::Headers(_) => None,
            _ => Some(self.first_line_lf_idx + 1 .. self.last_header_lf_idx + 1)
        }
    }
    pub fn get_body_start_index(&self) -> Option<usize> {
        match self.stage {
            Stage::Finished(Err(())) => None,
            Stage::FirstLine | Stage::Headers(_)  => None,
            _ => Some(self.last_header_lf_idx + 1)
        }
    }
}

