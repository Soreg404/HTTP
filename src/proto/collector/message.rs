use super::glue::*;
use crate::proto::consts::Version;
use super::collect_error::CollectError;
use crate::proto::state_reader::{ StateReader, Poll };
use crate::proto::header_parser::HeaderBodyParser;

#[derive(Default)]
pub struct Message<T>
where T: Subtype {
    state: CollectState,
    stage: CollectStage,
    buffer: Vec<u8>,
    buffer_reader: StateReader,
    specific: T,
    incomplete: MessageIncomplete
}

#[derive(Default, Debug)]
enum CollectState {
    #[default]
    Processing,
    Finished(Result<(), CollectError>)
}

// tmp Clone, should be Copy but until I deal with that Vec<u8>
#[derive(Default, Debug, Clone)]
enum CollectStage {
    #[default]
    FirstLine,
    MainHeaders,
    AfterMainHeaders,
    MainBody {
        content_length: usize
    },
    Attachments {
        content_length: usize,
        boundary: Vec<u8>, // temporary - should be Rdx
        stage: AttachmentCollectStage
    }
}

#[derive(Debug, Copy, Clone)]
enum AttachmentCollectStage {
    Skip,
    Headers,
    AfterHeaders,
    Content
}

// todo: avoid malloc
#[derive(Default)]
struct MessageIncomplete {
    version: Version,
    headers: Vec<Vec<u8>>,

    h_content_length: Option<usize>,
    multipart_boundary: Option<Vec<u8>>,

    body: Vec<u8>,
    attachments: Vec<Attachment>
}

// todo: temp, later change to Rdx and avoid malloc
#[derive(Default)]
struct Attachment {
    disp_name: Option<Vec<u8>>,
    disp_filename: Option<Vec<u8>>,
    content: Vec<u8>
}

enum AdvanceResult {
    Pending,
    Continue,
    ChangeStage(CollectStage),
    Finished(Result<(), CollectError>)
}

impl<T> Message<T>
where T: Subtype + Default {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        macro_rules! dtrace1 { ($msg:expr) => {
            dtrace!("push_bytes()", $msg); 
        } }
        dtrace1!("begin");

        // temporary
        self.buffer.extend_from_slice(bytes);
        ////

        if let CollectStage::FirstLine = self.stage {
            dtrace1!("collect FirstLine");
            match self.buffer_reader.take_line(
                &self.buffer
            ) {
                Poll::Pending => {
                    dtrace1!("FirstLine pending");
                    return
                }
                Poll::Ready(line_rdx) => {
                    match self.specific.first_line(
                        line_rdx.get(&self.buffer),
                        &mut self.incomplete.version 
                    ) {
                        Ok(()) => {
                            self.stage = CollectStage::MainHeaders;
                        },
                        Err(e) => {
                            self.state = CollectState::Finished(Err(e));
                            return;
                        }
                    }
                }
            }
        } 
        loop {
            dtrace1!("loop advance");
            match self.incomplete.advance(
                &self.buffer,
                &mut self.buffer_reader,
                self.stage.clone()
            ) {
                AdvanceResult::Pending => {
                    dtrace1!("Pending");
                    break
                }
                AdvanceResult::Continue => {
                    dtrace1!("Continue");
                    continue
                }
                AdvanceResult::ChangeStage(s) => {
                    dtrace1!(format!("StateChange: {:?}", s));
                    self.stage = s;
                    continue;
                }
                AdvanceResult::Finished(r) => {
                    dtrace1!(format!("Finished: {:?}", r));
                    self.state = CollectState::Finished(r);
                    break;
                }
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
