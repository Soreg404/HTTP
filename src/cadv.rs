use super::*;
use crate::helpers::*;

impl Collector {
    pub fn advance_inner(&mut self, bytes: &[u8]) -> Advance {
        trace!("advance start; proc_bytes={}; current_bytes={:?}",
            self.proc_bytes,
            String::from_utf8_lossy(bytes),
        );
        match self.stage {
            Stage::Finished(_) => panic!("already finished"),
            _ => {}
        }
        let mut i = 0;
        let mut _dbg_last_i = 0;
        let mut av_action = AvAction::Nop;
        'advance_loop: loop {
            trace!("advance loop;     i={i:05}; c={:?}; stage={:?}; \
            adv_buf={:?}; consumed={:?}; rest={:?}{};",
                bytes.get(i).map(|v| *v as char),
                self.stage,
                String::from_utf8_lossy(&self.advance_buffer[..self.advance_buffer_head]),
                String::from_utf8_lossy(&bytes[_dbg_last_i..i]),
                String::from_utf8_lossy(&bytes[i..std::cmp::min(bytes.len(), i + 20)]),
                if i + 20 < bytes.len() { "..." } else { "" }
            );
            _dbg_last_i = i;
            if i == bytes.len() {
                break;
            }
            match &mut self.stage {
                Stage::Finished(_) => unreachable!(),
                Stage::FirstLine => {
                    while i < bytes.len() {
                        if bytes[i] == b'\n' {
                            let idx = self.proc_bytes + i;
                            self.first_line_lf_idx = idx;
                            self.stage = Stage::Headers(StageHeaders::HeaderName);
                            av_action = AvAction::FirstLineReady(0..idx + 1);
                            i += 1;
                            break 'advance_loop;
                        }
                        i += 1;
                    }
                }
                Stage::Headers(StageHeaders::HeaderName) => {
                    while i < bytes.len() {
                        let name = &self.advance_buffer[0..self.advance_buffer_head];
                        if bytes[i] == b':' {
                            if name.eq_ignore_ascii_case(b"content-length") {
                                self.stage = Stage::Headers(StageHeaders::ContentLengthHeaderValue);
                            } else if name.eq_ignore_ascii_case(b"transfer-encoding") {
                                self.stage = Stage::Headers(StageHeaders::TransferEncodingHeaderValue);
                            } else {
                                self.stage = Stage::Headers(StageHeaders::HeaderValue);
                            }
                            self.advance_buffer_head = 0;
                            i += 1;
                            continue 'advance_loop;
                        }
                        else if bytes[i] == b'\n' {
                            if !name.trim_ascii().is_empty() {
                                self.stage = Stage::Finished(Err(()));
                            } else {
                                let idx = self.proc_bytes + i;
                                self.last_header_lf_idx = idx;
                                match self.content_length {
                                    Some(v) => {
                                        if let Some(true) = self.transfer_encoding {
                                            self.stage = Stage::Finished(Err(()));
                                        } else {
                                            self.stage = Stage::BodyNormal(v);
                                        }
                                    }
                                    None => {
                                        if let Some(true) = self.transfer_encoding {
                                            self.stage = Stage::BodyChunkLength;
                                        } else {
                                            self.stage = Stage::Finished(Ok(()));
                                        }
                                    }
                                }
                                av_action = AvAction::HeadersReady(
                                    self.first_line_lf_idx + 1 .. idx + 1);
                            }
                            self.advance_buffer_head = 0;
                            i += 1;
                            break 'advance_loop;
                        }
                        if self.advance_buffer_head < self.advance_buffer.len() {
                            self.advance_buffer[self.advance_buffer_head] = bytes[i];
                            self.advance_buffer_head += 1;
                        }
                        i += 1;
                    }
                }
                Stage::Headers(StageHeaders::HeaderValue) => {
                    while i < bytes.len() {
                        if bytes[i] == b'\n' {
                            self.stage = Stage::Headers(StageHeaders::HeaderName);
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                }
                Stage::Headers(StageHeaders::ContentLengthHeaderValue) => {
                    if self.content_length.is_some() {
                        self.stage = Stage::Finished(Err(()));
                        break;
                    }
                    while i < bytes.len() && bytes[i] != b'\n' {
                        if self.advance_buffer_head < self.advance_buffer.len() {
                            self.advance_buffer[self.advance_buffer_head] = bytes[i];
                            self.advance_buffer_head += 1;
                        } else {
                            self.stage = Stage::Finished(Err(()));
                            break;
                        }
                        i += 1;
                    }
                    if i == bytes.len() {
                        break 'advance_loop;
                    }
                    i += 1;
                    let value_str = self.advance_buffer[..self.advance_buffer_head].trim_ascii();
                    let cl = usize_from_u8_slice(value_str);
                    match cl {
                        Some(v) => {
                            trace!("content-length header value: {v}");
                            self.content_length = Some(v);
                            self.advance_buffer_head = 0;
                            self.stage = Stage::Headers(StageHeaders::HeaderName);
                        }
                        None => {
                            self.stage = Stage::Finished(Err(()));
                        }
                    }
                }
                Stage::Headers(StageHeaders::TransferEncodingHeaderValue) => {
                    if self.transfer_encoding.is_some() {
                        self.stage = Stage::Finished(Err(()));
                        break;
                    }
                    while i < bytes.len() && bytes[i] != b'\n' {
                        if self.advance_buffer_head < self.advance_buffer.len() {
                            self.advance_buffer[self.advance_buffer_head] = bytes[i];
                            self.advance_buffer_head += 1;
                        } else {
                            self.stage = Stage::Finished(Err(()));
                            break;
                        }
                        i += 1;
                    }
                    if i == bytes.len() {
                        break 'advance_loop;
                    }
                    let val_str = self.advance_buffer[..self.advance_buffer_head]
                        .trim_ascii();
                    self.transfer_encoding = Some(
                        val_str.eq_ignore_ascii_case(b"chunked")
                    );
                    self.advance_buffer_head = 0;
                    i += 1;
                    self.stage = Stage::Headers(StageHeaders::HeaderName);
                }
                Stage::BodyNormal(l) => {
                    let l = *l;
                    let body_start = self.last_header_lf_idx + 1;
                    let current_body_len = self.proc_bytes - body_start;
                    if current_body_len + bytes.len() >= l {
                        i = l - current_body_len;
                        let idx = self.proc_bytes + i;
                        self.stage = Stage::Finished(Ok(()));
                        av_action = AvAction::BodyReady(body_start..idx);
                    } else {
                        i = bytes.len();
                    }
                    break 'advance_loop;
                }
                Stage::BodyChunkLength => {
                    trace!("BodyChunkLength continues at proc_bytes={}; i={i:05}",
                        self.proc_bytes);
                    while i < bytes.len() && bytes[i] != b'\n' {
                        if self.advance_buffer_head < self.advance_buffer.len() {
                            self.advance_buffer[self.advance_buffer_head] = bytes[i];
                            self.advance_buffer_head += 1;
                        } else {
                            self.stage = Stage::Finished(Err(()));
                            break;
                        }
                        i += 1;
                    }
                    if i == bytes.len() {
                        break 'advance_loop;
                    }
                    let val_str = self.advance_buffer[0..self.advance_buffer_head]
                        .trim_ascii();
                    match usize_from_u8_slice_hex(val_str) {
                        Some(l) => {
                            if l == 0 {
                                self.stage = Stage::Finished(Ok(()));
                                break 'advance_loop;
                            } else {
                                self.advance_buffer_head = 0;
                                i += 1;
                                self.chunk_start_idx = self.proc_bytes + i;
                                self.stage = Stage::BodyChunk(l);
                            }
                        }
                        None => {
                            self.stage = Stage::Finished(Err(()));
                        }
                    }
                }
                Stage::BodyChunk(l) => {
                    let l = *l;
                    let current_chunk_len = (self.proc_bytes + i) - self.chunk_start_idx;
                    if current_chunk_len + (bytes.len() - i) >= l {
                        i += l - current_chunk_len;
                        let idx = self.proc_bytes + i;
                        self.advance_buffer_head = 0;
                        self.stage = Stage::BodyChunkSkipCRLF;
                        av_action = AvAction::BodyChunkReady(self.chunk_start_idx..idx);
                    } else {
                        i = bytes.len();
                    }
                    break 'advance_loop;
                }
                Stage::BodyChunkSkipCRLF => {
                    let h = &mut self.advance_buffer_head;
                    match bytes[i] {
                        b'\r' => {
                            if *h == 0 {
                                i += 1;
                                *h = 1;
                                continue 'advance_loop;
                            }
                        }
                        b'\n' => {
                            *h = 0;
                            i += 1;
                            self.stage = Stage::BodyChunkLength;
                            continue 'advance_loop;
                        }
                        _ => {}
                    }
                    self.stage = Stage::Finished(Err(()));
                }
            }
        }
        trace!("advance loop end; i={i:05}; c={:?}; stage={:?};\n    \
            adv_buf={:?}; bytes_consumed={:?}; bytes_rest={:?}; av_action={:?}",
            bytes.get(i).map(|v| *v as char),
            self.stage,
            String::from_utf8_lossy(&self.advance_buffer[..self.advance_buffer_head]),
            String::from_utf8_lossy(&bytes[..i]),
            String::from_utf8_lossy(&bytes[i..]),
            av_action,
        );
        self.proc_bytes += i;
        Advance {
            current: i,
            total: self.proc_bytes,
            av_action
        }
    }
}
