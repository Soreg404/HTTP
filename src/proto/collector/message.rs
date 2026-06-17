use super::glue::*;
use crate::proto::consts::Version;
use super::collect_error::CollectError;
use crate::proto::state_reader::{ StateReader, Poll };
use crate::proto::header_parser::HeaderBodyParser;

fn strip_crlf(buffer: &[u8]) -> Option<&[u8]> {
    let s = buffer;
    let s = match s.last() {
        Some(b'\n') => &s[..s.len() - 1],
        _ => return None
    };
    let s = match s.last() {
        Some(b'\r') => &s[..s.len() - 1],
        _ => s
    };
    Some(s)
}

#[derive(Default)]
pub struct Message {
    state: CollectState,
    stage: CollectStage,
    buffer: Vec<u8>,
    read_base: usize,
    incomplete: MessageIncomplete
}

#[derive(Default, Debug)]
enum CollectState {
    #[default]
    Processing,
    Finished(Result<(), CollectError>)
}

#[derive(Debug, Clone)]
enum CollectStage {
    RequestMethod,
    RequestTarget,
    RequestVersion,

    ResponseVersion,
    ResponseCode,
    ResponseDesc,

    MainHeaders,
    AfterMainHeaders,
    Body {
        start_index: usize,
        content_length: usize
    }
}

#[derive(Default)]
struct MessageIncomplete {
    version: Version,

    multipart_boundary: Option<Rdx>,
    content_length: Option<usize>,
}

impl Message {
    pub fn new_request() -> Self {
        Self {
            collect_stage: 
            ..Default::default()
        }
    }
    pub fn new_response() -> Self {
        Self {
            collect_stage: 
            ..Default::default()
        }
    }
    pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
        macro_rules! dtrace1 { ($msg:expr) => {
            dtrace!("push_bytes()", $msg); 
        } }
        dtrace1!("begin");

        let mut bytes_i = 0;
        while bytes_i < bytes.len() {
            // todo: match CollectStage::Body and check if length is ok
            // return err if not
            self.buffer.push(bytes[i]);

            self.advance3();

            if self.is_finished() {
                break
            }
        }
        
        bytes_i
    }
    fn advance3(&mut self) {
        let s = &self.buffer[self.read_base..];
        match self.stage {
            CollectStage::RequestMethod => {
            }
        }
    }
    pub fn is_finished(&self) -> bool {
        match self.state {
            CollectState::Processing => false,
            CollectState::Finished(_) => true
        }
    }
}

impl MessageIncomplete {
    
    fn advance(
        &mut self,
        buffer: &[u8],
        buffer_reader: &mut StateReader,
        stage: CollectStage,
    ) -> AdvanceResult {
        match stage {
            CollectStage::FirstLine => unreachable!(),
            CollectStage::MainHeaders => {
                let line_rdx = match buffer_reader.take_line(buffer) {
                    Poll::Pending => return AdvanceResult::Pending,
                    Poll::Ready(v) => v
                };

                let line_bytes = line_rdx.get(&buffer);
                dtrace!("MainHeaders", format!("processing line: {:?}",
                    String::from_utf8_lossy(line_bytes)));

                // todo: safety checks on line length and allowed characters

                if line_bytes.trim_ascii().is_empty() {
                    if !line_bytes.is_empty() {
                        return AdvanceResult::Finished(Err(
                                CollectError::TBD("invalid empty header line".to_string())));
                    }
                    dtrace!("Mainheaders", "empty header line, \
                        change stage to AfterMainHeaders");
                    return AdvanceResult::ChangeStage(
                        CollectStage::AfterMainHeaders);
                }

                // todo: pass line_rdx and remove mallocs
                match self.process_main_header_line(line_bytes) {
                    Ok(()) => {},
                    Err(e) => return AdvanceResult::Finished(Err(e))
                }

                // todo: avoid malloc
                self.headers.push(line_bytes.to_vec());

                dtrace!("MainHeaders", "continue MainHeaders");
                AdvanceResult::Continue
            },
            CollectStage::AfterMainHeaders => {
                dtrace!("AfterMainHeaders", "begin");
                match self.h_content_length {
                    None => AdvanceResult::Finished(Ok(())),
                    Some(l) => {
                        AdvanceResult::ChangeStage(
                            match &self.multipart_boundary {
                                None => CollectStage::MainBody {
                                    content_length: l
                                },
                                Some(b) => CollectStage::Attachments {
                                    content_length: l,
                                    boundary: b.to_vec(), // todo: should be Rdx
                                    stage: AttachmentCollectStage::Skip
                                }
                            }
                        )
                    }
                }
            }
            CollectStage::MainBody { content_length } => {
                match buffer_reader.take_exact(buffer, content_length) {
                    Poll::Pending => return AdvanceResult::Pending,
                    Poll::Ready(rdx) => {
                        // todo: avoid malloc
                        self.body = rdx.get(buffer).to_vec();
                        AdvanceResult::Finished(Ok(()))
                    }
                }
            }
            CollectStage::Attachments {
                content_length,
                boundary,
                stage
            } => match stage {
                AttachmentCollectStage::Skip => {
                    dtrace!("Attachment->Skip", "begin");
                    match buffer_reader.take_attachment(buffer, &boundary) {
                        Poll::Pending => AdvanceResult::Pending,
                        Poll::Ready(boundary_info) => {
                            if boundary_info.is_last {
                                AdvanceResult::Finished(Ok(()))
                            } else {
                                self.attachments.push(Attachment::default());
                                AdvanceResult::ChangeStage(
                                    CollectStage::Attachments {
                                        content_length,
                                        boundary,
                                        stage: AttachmentCollectStage::Headers
                                    }
                                )
                            }
                        }
                    }
                }
                AttachmentCollectStage::Headers => {
                    macro_rules! dtrace1 { ($msg:expr) => {
                        dtrace!("Attachment->Headers", $msg)
                    } }
                    let line_rdx = match buffer_reader.take_line(buffer) {
                        Poll::Pending => return AdvanceResult::Pending,
                        Poll::Ready(v) => v
                    };

                    let line_bytes = line_rdx.get(&buffer);
                    dtrace1!(format!("processing line: {:?}",
                            String::from_utf8_lossy(line_bytes)));

                    // todo: safety checks on line length and allowed characters

                    if line_bytes.trim_ascii().is_empty() {
                        if !line_bytes.is_empty() {
                            return AdvanceResult::Finished(Err(
                                    CollectError::TBD("invalid empty header line".to_string())));
                        }
                        dtrace1!("empty header line, \
                            change stage to AfterHeaders");
                        return AdvanceResult::ChangeStage(
                            CollectStage::Attachments {
                                content_length,
                                boundary,
                                stage: AttachmentCollectStage::AfterHeaders
                            });
                    }

                    AdvanceResult::Continue
                }
                AttachmentCollectStage::AfterHeaders => {
                    dtrace!("Attachment->AfterHeaders", "begin");
                    if self.attachments.last().unwrap().disp_name.is_none() {
                        AdvanceResult::Finished(Err(CollectError::TBD(
                                    "attachment missing name".to_string())))
                    } else {
                        AdvanceResult::ChangeStage(
                            CollectStage::Attachments {
                                content_length,
                                boundary,
                                stage: AttachmentCollectStage::Content
                            }
                        )
                    }
                }
                AttachmentCollectStage::Content => {
                    dtrace!("Attachment->Content", "begin");
                    match buffer_reader.take_attachment(buffer, &boundary) {
                        Poll::Pending => AdvanceResult::Pending,
                        Poll::Ready(boundary_info) => {
                            if boundary_info.is_last {
                                AdvanceResult::Finished(Ok(()))
                            } else {
                                self.attachments.last_mut().unwrap()
                                    .content = boundary_info.content.get(buffer)
                                    // todo: temporary, avoid malloc
                                    .to_vec();
                                AdvanceResult::ChangeStage(
                                    CollectStage::Attachments {
                                        content_length,
                                        boundary,
                                        stage: AttachmentCollectStage::Headers
                                    }
                                )
                            }
                        }
                    }
                }
            }
            s => AdvanceResult::Finished(Err(CollectError::TBD(format!(
                            "unimplemented: {s:?}"))))
        }
    }
    fn process_main_header_line(
        &mut self,
        line_bytes: &[u8]
    ) -> Result<(), CollectError> {
        macro_rules! dtrace1 { ($msg:expr) => {
            dtrace!("MainHeaders->process_line()", $msg) }}
        dtrace1!("begin");

        let header_rdx = match crate::proto::header_parser::header_from_line(line_bytes) {
            Ok(v) => v,
            Err(()) => return Err(CollectError::TBD("invalid header, \
                    failed to get header_from_line".to_string()))
        };
        let header_name_bytes = header_rdx.name.get(line_bytes);
        let header_body_bytes = header_rdx.body.get(line_bytes);
        dtrace1!(format!("got header {:?}:{:?}",
                String::from_utf8_lossy(header_name_bytes),
                String::from_utf8_lossy(header_body_bytes)));

        if header_name_bytes.eq_ignore_ascii_case(b"content-length") {
            dtrace1!("parse content length begin");
            self.h_content_length = match su8_to_dec(
                header_body_bytes.trim_ascii()) {
                Ok(v) => Some(v),
                Err(()) => return Err(CollectError::TBD(
                        "invalid content length header".to_string()))
            };
            dtrace1!(format!("got content-length: {:?}", self.h_content_length));
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
            let ctm = c_type.main.get(header_body_bytes);
            let cts = c_type.sub.get(header_body_bytes);
            dtrace1!(format!("got type/subtype: {:?}/{:?}",
                    String::from_utf8_lossy(ctm),
                    String::from_utf8_lossy(cts)));
            if ctm == b"multipart" && cts == b"form-data" {
                // first attr has to be "boundary"
                let attr_value_rdx = {
                    let e = Err(CollectError::TBD(
                            "missing or invalid boundary attribute".to_string()));
                    match p.next_attribute() {
                        Some(Ok(v)) => {
                            if v.key.get(header_body_bytes) != b"boundary" {
                                return e;
                            }
                            v.value
                        }
                        _ => return e
                    }
                };

                let boundary = attr_value_rdx.get(header_body_bytes);
                dtrace1!(format!("got boundary: {:?}",
                        String::from_utf8_lossy(boundary)));
                // todo: avoid malloc
                self.multipart_boundary = Some(boundary.to_vec());
            }
        }

        Ok(())
    }
    fn process_attachment_header_line(
        &mut self,
        line_bytes: &[u8]
    ) -> Result<(), CollectError> {
        macro_rules! dtrace1 { ($msg:expr) => {
            dtrace!("AttachmentHeaders->process_line()", $msg) }}
        dtrace1!("begin");

        let header_rdx = match crate::proto::header_parser::header_from_line(line_bytes) {
            Ok(v) => v,
            Err(()) => return Err(CollectError::TBD("invalid header, \
                    failed to get header_from_line".to_string()))
        };
        let header_name_bytes = header_rdx.name.get(line_bytes);
        let header_body_bytes = header_rdx.body.get(line_bytes);
        dtrace1!(format!("got header {:?}:{:?}",
                String::from_utf8_lossy(header_name_bytes),
                String::from_utf8_lossy(header_body_bytes)));

        let current_attachment = self.attachments.last_mut().unwrap();

        if header_name_bytes.eq_ignore_ascii_case(b"content-disposition") {
            let name = &mut current_attachment.disp_name;
            let filename = &mut current_attachment.disp_filename;
            *name = Some(b"abc".to_vec());
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
