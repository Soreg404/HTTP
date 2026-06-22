use crate::proto::consts::*;
use super::collect_error::CollectError;
use crate::proto::state_reader::StateReader;
use crate::proto::header_parser::{ header_from_line, HeaderBodyParser };
use super::parser;
use index_slice::IndexSlice;

#[derive(Default)]
pub struct Message {
    state: CollectState,
    stage: CollectStage,
    buffer: Vec<u8>,
    buffer_reader: StateReader,

    i_request: IncompleteRequest,
    i_response: IncompleteResponse,
    i_message: IncompleteMessage
}

#[derive(Default, Debug)]
enum CollectState {
    #[default]
    Processing,
    Finished(Result<(), CollectError>)
}

#[derive(Default, Debug, Clone)]
enum CollectStage {
    #[default]
    Unstarted,

    RequestFirstLine,
    ResponseFirstLine,

    Headers,
    Body {
        content_length: usize,
        start_index: usize,
    }
}

#[derive(Default)]
struct IncompleteRequest {
    method: Option<Method>,
    target: Option<Vec<u8>>,
}
#[derive(Default)]
struct IncompleteResponse {
    code: Option<StatusCode>,
    desc: IndexSlice
}
#[derive(Default)]
struct IncompleteMessage {
    version: Version,

    headers: Vec<IndexSlice>,

    multipart_boundary: Option<IndexSlice>,
    content_length: Option<usize>,

    body: IndexSlice
}

impl Message {
    pub fn new_request() -> Self {
        Self {
            stage: CollectStage::RequestFirstLine,
            ..Default::default()
        }
    }
    pub fn new_response() -> Self {
        Self {
            stage: CollectStage::ResponseFirstLine,
            ..Default::default()
        }
    }
    pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
        macro_rules! dtrace1 { ($msg:expr) => {
            dtrace!("push_bytes()", $msg); 
        } }
        dtrace1!("begin");

        let start_head = self.buffer_reader.head;

        // todo: match CollectStage::Body and check if length is ok
        // return err if not
        self.buffer.extend_from_slice(bytes);

        // note: potentially, loop while buffer_reader.head < buffer.len

        self.advance3();

        let bytes_processed = self.buffer_reader.head - start_head;
        dtrace1!(format!("processed {bytes_processed} out of {} bytes", bytes.len()));
        bytes_processed
    }
    fn advance3(&mut self) {
        loop {
            match self.stage {
                CollectStage::Unstarted => unreachable!(),
                CollectStage::RequestFirstLine => {
                    match self.buffer_reader.take_line(&self.buffer) {
                        None => return,
                        Some(line_rdx) => {
                            let line = line_rdx.as_slice_of(&self.buffer);
                            match parser::first_line_request(line) {
                                Err(e) => { 
                                    self.raise_error(e);
                                    return;
                                }
                                Ok(v) => {
                                    dtrace!("RequestFirstLine", format!(" v={v:?}"));
                                    self.i_request.method = Some(v.method);
                                    self.i_request.target = Some(v.target);

                                    self.i_message.version = v.version;

                                    self.stage = CollectStage::Headers;
                                    continue;
                                }
                            }
                        }
                    }
                }
                CollectStage::ResponseFirstLine => todo!(),
                CollectStage::Headers => {
                    let (line, line_rdx) = match self.buffer_reader.take_line(&self.buffer) {
                        None => return,
                        Some(v) => (v.as_slice_of(&self.buffer), v)
                    };

                    if line.trim_ascii().is_empty() {
                        if !line.is_empty() {
                            self.raise_error(
                                CollectError::TBD(
                                    "invalid header line: empty header line has whitespace"
                                    .to_string()
                                )
                            );
                            return;
                        }

                        match self.i_message.content_length {
                            None => {
                                self.state = CollectState::Finished(Ok(()));
                                return;
                            }
                            Some(content_length) => {
                                self.stage = CollectStage::Body {
                                    content_length,
                                    start_index: self.buffer_reader.base
                                };
                                continue;
                            }
                        }
                    }

                    self.i_message.process_main_header_line(line);
                }
                CollectStage::Body {
                    content_length, ..
                } => {
                    dtrace!("Body", format!("begin, length: {content_length:?}"));
                    match self.buffer_reader.take_exact(&self.buffer, content_length) {
                        None => {
                            dtrace!("Body", format!("return, read-len: {:?}",
                                    self.buffer_reader.head - self.buffer_reader.base)); 
                            return;
                        },
                        Some(idx) => {
                            dtrace!("Body", format!("done, length: {content_length:?}"));
                            self.i_message.body = idx;
                            self.state = CollectState::Finished(Ok(()));
                            return;
                        }
                    }
                }
            }
        }
    }
    fn raise_error(&mut self, e: CollectError) {
        self.state = CollectState::Finished(Err(e));
    }
    pub fn is_finished(&self) -> bool {
        match self.state {
            CollectState::Processing => false,
            CollectState::Finished(_) => true
        }
    }
}

impl IncompleteMessage {
    fn process_main_header_line(
        &mut self,
        line_bytes: &[u8]
    ) -> Result<(), CollectError> {
        macro_rules! dtrace1 { ($msg:expr) => {
            dtrace!("process_main_header_line()", $msg) }}
        dtrace1!("begin");

        let header_rdx = match header_from_line(line_bytes) {
            Ok(v) => v,
            Err(()) => return Err(CollectError::TBD("invalid header, \
                    failed to get header_from_line".to_string()))
        };
        let header_name_bytes = header_rdx.name.as_slice_of(line_bytes);
        let header_body_bytes = header_rdx.body.as_slice_of(line_bytes);
        dtrace1!(format!("got header {:?}:{:?}",
                String::from_utf8_lossy(header_name_bytes),
                String::from_utf8_lossy(header_body_bytes)));

        if header_name_bytes.eq_ignore_ascii_case(b"content-length") {
            dtrace1!("parse content length begin");
            self.content_length = match su8_to_dec(
                header_body_bytes.trim_ascii()) {
                Ok(v) => Some(v),
                Err(()) => return Err(CollectError::TBD(
                        "invalid content length header".to_string()))
            };
            dtrace1!(format!("got content-length: {:?}", self.content_length));
        } else if header_name_bytes.eq_ignore_ascii_case(b"content-type") {
            // Extract Optional multipart boundary
            dtrace1!("parse content type begin");

            // get and check content-type: main/sub == "multipart/form-data"
            let mut p = HeaderBodyParser::new(header_body_bytes); 
            let c_type = match p.next_type() {
                Err(()) => return Err(CollectError::TBD(
                        "missing or invalid type".to_string())),
                Ok(v) => v,
            };
            let ctm = c_type.main.as_slice_of(header_body_bytes);
            let cts = c_type.sub.as_slice_of(header_body_bytes);
            dtrace1!(format!("got type/subtype: {:?}/{:?}",
                    String::from_utf8_lossy(ctm),
                    String::from_utf8_lossy(cts)));
            if ctm == b"multipart" && cts == b"form-data" {
                // first attr has to be "boundary"
                let attr_value_idx = {
                    let e = Err(CollectError::TBD(
                            "missing or invalid boundary attribute".to_string()));
                    match p.next_attribute() {
                        Some(Ok(v)) => {
                            if v.key.as_slice_of(header_body_bytes) != b"boundary" {
                                return e;
                            }
                            v.value
                        }
                        _ => return e
                    }
                };

                let boundary = attr_value_idx.as_slice_of(header_body_bytes);
                dtrace1!(format!("got boundary: {:?}",
                        String::from_utf8_lossy(boundary)));
                // todo: avoid malloc
                self.multipart_boundary = Some(attr_value_idx);
            }
        }

        Ok(())
    }
}

fn su8_to_dec(s: &[u8]) -> Result<usize, ()> {
	let mut v = 0usize;
	for c in s {
		if !c.is_ascii_digit() {
			return Err(());
		}
		v = v * 10 + (c - b'0') as usize;
	}
	Ok(v)
}
