use crate::consts::{Method, StatusCode, Version};
use crate::proto::message_common::CollectMessageState::Ended;
use crate::proto::parser::ParseError::{InvalidHeader, RepeatedHeader};
use crate::proto::parser::{parse_header_line, HeaderLineParseResult, ParseError};
use crate::proto::state_buffer_reader::{BufferReader, StateBufferReaderResult};
use std::cmp::min;
use CollectMessageState::Processing;
use ProcessingStage::{Body, FirstLine, PreludeHeaders};
use StateBufferReaderResult::{Done, NotEnoughBytes};

#[derive(Debug, Eq, PartialEq, Clone)]
enum ProcessingStage {
	FirstLine,
	PreludeHeaders,
	Body,
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum CollectMessageState {
	Ended(Result<(), ParseError>),
	Processing(ProcessingStage),
}

#[derive(Debug)]
pub struct MessageCommon<T>
where
	T: MessageSpecific,
{
	message_specific: T,

	version: Option<Version>,
	headers: Vec<(Vec<u8>, Vec<u8>)>,

	buffer1: Vec<u8>,
	buffer1_reader: BufferReader,

	collect_state: CollectMessageState,

	content_length: Option<usize>,
	multipart_boundary: Option<Vec<u8>>,

	body_bytes: Vec<u8>,
}

trait MessageSpecific {
	fn collect_first_line(
		&mut self,
		line: &[u8],
	) -> Result<Version, ParseError>;
}

enum ProcessLoopOutcome {
	NotEnoughBytes,
	Continue,
}

impl<T> MessageCommon<T>
where
	T: MessageSpecific,
{
	pub fn parse_result(&self) -> Option<Result<(), ParseError>> {
		match self.collect_state {
			Ended(Ok(())) => Some(Ok(())),
			Ended(Err(e)) => Some(Err(e)),
			Processing(_) => None
		}
	}

	fn new_common(t: T) -> Self {
		Self {
			message_specific: t,

			version: None,
			headers: Vec::new(),
			buffer1: Vec::new(),
			buffer1_reader: BufferReader::new(),
			collect_state: Processing(FirstLine),

			content_length: None,
			multipart_boundary: None,

			body_bytes: Vec::new(),
		}
	}

	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
		self.buffer1.extend_from_slice(bytes);
		self.process_loop();

		let leftover_bytes =
			self.buffer1.len() - self.buffer1_reader.current_read_head();
		bytes.len() - min(leftover_bytes, bytes.len())
	}


	fn process_loop(&mut self) {
		loop {
			match self.collect_state.clone() {
				Ended(_) => {
					return;
				}
				Processing(stage) => {
					if !self.stage_match(stage) {
						return;
					}
				}
			}
		}
	}

	fn stage_match(&mut self, stage: ProcessingStage) -> bool {
		match stage {
			FirstLine =>
				match self.buffer1_reader
					.take_line(&self.buffer1) {
					NotEnoughBytes => false,
					Done(line) => {
						match self.message_specific
							.collect_first_line(line) {
							Ok(ver) => {
								self.version = Some(ver);
								self.collect_state = Processing(PreludeHeaders);
								true
							}
							Err(e) => {
								self.collect_state = Ended(Err(e));
								false
							}
						}
					}
				},

			PreludeHeaders => {
				match self.buffer1_reader
					.take_line(&self.buffer1) {
					NotEnoughBytes => false,
					Done(header_line) => {
						match parse_header_line(header_line) {
							HeaderLineParseResult::Empty => {
								self.collect_state = Processing(Body);
								true
							}
							HeaderLineParseResult::Ok {
								field_name,
								field_value
							} => {
								self.headers.push(
									(field_name.into(), field_value.into()));

								if field_name.eq_ignore_ascii_case(b"content-length") {
									if self.content_length.is_some() {
										self.collect_state = Ended(Err(RepeatedHeader));
										return false;
									}
									match String::from_utf8_lossy(field_value).parse::<usize>() {
										Ok(v) => self.content_length = Some(v),
										Err(_e) => {
											self.collect_state = Ended(Err(InvalidHeader));
											return false;
										}
									}
								} else if field_name.eq_ignore_ascii_case(b"content-type") {
									// todo: This is a terrible bodge pls do better

									let multipart_value = b"multipart/form-data";
									if field_value[..multipart_value.len()]
										.eq_ignore_ascii_case(multipart_value) {
										if self.multipart_boundary.is_some() {
											self.collect_state = Ended(Err(RepeatedHeader));
											return false;
										}
										let rest = field_value[multipart_value.len()..].trim_ascii_start();
										if rest.len() < 10 || rest[0] != b';' {
											self.collect_state = Ended(Err(InvalidHeader));
											return false;
										}
										let rest = rest[1..].trim_ascii_start();
										if rest[..9].eq_ignore_ascii_case(b"boundary=") {
											self.multipart_boundary = Some(rest[9..].into());
										} else {
											self.collect_state = Ended(Err(InvalidHeader));
											return false;
										}
									}
								}

								true
							}
							HeaderLineParseResult::Err(e) => {
								self.collect_state = Ended(Err(e));
								false
							}
						}
					}
				}
			}
			Body => {
				match self.content_length {
					None => {
						self.collect_state = Ended(Ok(()));
						false
					}
					Some(v) => {
						match self.buffer1_reader.take_exact(&self.buffer1, v) {
							NotEnoughBytes => false,
							Done(s) => {
								self.body_bytes.extend_from_slice(s);
								self.collect_state = Ended(Ok(()));
								false
							}
						}
					}
				}
			}
		}
	}
}

#[derive(Debug)]
pub struct RequestSpecific {
	method: Option<Method>,
	url: Option<Vec<u8>>,
}

impl MessageSpecific for RequestSpecific {
	fn collect_first_line(&mut self, line: &[u8]) -> Result<Version, ParseError> {
		let result =
			crate::proto::parser::parse_request_first_line(line)?;

		self.method = Some(result.method);
		self.url = Some(result.url_slice);

		Ok(result.version)
	}
}

#[derive(Debug)]
pub struct ResponseSpecific {
	status_code: Option<StatusCode>,
	status_desc: Option<String>,
}

impl MessageSpecific for ResponseSpecific {
	fn collect_first_line(&mut self, line: &[u8]) -> Result<Version, ParseError> {
		let result =
			crate::proto::parser::parse_response_first_line(line)?;

		self.status_code = Some(result.status_code);
		self.status_desc = Some(result.status_desc);

		Ok(result.version)
	}
}

pub type RequestCollector = MessageCommon<RequestSpecific>;
pub type ResponseCollector = MessageCommon<ResponseSpecific>;

impl RequestCollector {
	pub fn new() -> Self {
		MessageCommon::<RequestSpecific>::new_common(
			RequestSpecific {
				method: None,
				url: None,
			}
		)
	}
}

impl ResponseCollector {
	pub fn new() -> Self {
		MessageCommon::<ResponseSpecific>::new_common(
			ResponseSpecific {
				status_code: None,
				status_desc: None,
			}
		)
	}
}

pub fn message_common_dbg<T>(msg: &MessageCommon<T>)
where
	T: MessageSpecific,
{
	for (h, v) in &msg.headers {
		println!("header: {:?}:{:?}",
				 String::from_utf8_lossy(h.as_slice()), String::from_utf8_lossy(v.as_slice()));
	}
	println!("content-length: {:?}", msg.content_length);
	println!("multipart boundary: {:?}", match &msg.multipart_boundary {
		None => None,
		Some(v) => Some(String::from_utf8_lossy(v.as_slice()))
	});
	println!();
	println!("body: ({} bytes)", msg.body_bytes.len());
	println!(
		"<<<{:.100}>>>{}",
		String::from_utf8_lossy(&msg.body_bytes),
		if msg.body_bytes.len() > 100 {
			format!("\n[+{} bytes]", msg.body_bytes.len() - 100)
		} else { String::new() }
	)
}
