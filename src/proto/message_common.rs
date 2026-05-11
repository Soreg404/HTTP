use crate::consts::{Method, StatusCode, Version};
use crate::proto::message_common::CollectMessageState::Ended;
use crate::proto::parser::ParseError;
use crate::proto::parser::ParseError::{HeaderLine, InvalidHeader, RepeatedContentLengthHeader};
use crate::proto::state_buffer_reader::{StateBufferReader, StateBufferReaderResult};
use CollectMessageState::Processing;
use ProcessingStage::{Body, FirstLine, PreludeHeaders};
use StateBufferReaderResult::{Done, NotEnoughBytes};

#[derive(Debug, Eq, PartialEq)]
enum ProcessingStage {
	FirstLine,
	PreludeHeaders,
	Body,
}

#[derive(Debug, Eq, PartialEq)]
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

	pub working_buffer: Vec<u8>,
	pub working_buffer_reader: StateBufferReader,

	collect_state: CollectMessageState,

	content_length: Option<usize>
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
	fn new(t: T) -> Self {
		Self {
			message_specific: t,

			version: None,
			headers: Vec::new(),
			working_buffer: Vec::new(),
			working_buffer_reader: StateBufferReader::new(),
			collect_state: Processing(FirstLine),

			content_length: None,
		}
	}

	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
		if let Ended(_) = self.collect_state {
			return 0;
		}

		self.working_buffer.extend_from_slice(bytes);

		let begin_read_head =
			self.working_buffer_reader.current_read_head();

		self.process_working_buffer();

		let end_read_head =
			self.working_buffer_reader.current_read_head();

		{
			self.working_buffer_reader
				.reset_rest(&mut self.working_buffer);

			self.working_buffer.truncate(
				self.working_buffer_reader.current_read_head()
			);
		}

		end_read_head - begin_read_head
	}

	fn process_working_buffer(&mut self) {
		loop {
			match &self.collect_state {
				Processing(stage) => {
					if self.process_loop(stage) {
						continue;
					} else {
						break;
					}
				}
				Ended(_) => unreachable!(),
			}
		}
	}

	fn process_loop(&mut self, stage: &ProcessingStage) -> bool {
		match stage {
			FirstLine =>
				match self.working_buffer_reader
					.take_line(&self.working_buffer) {
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
				match self.working_buffer_reader
					.take_line(&self.working_buffer) {
					NotEnoughBytes => false,
					Done(header_line) => {
						let mut split_idx = None;

						for (i, c) in header_line.iter().enumerate() {
							if *c < b' ' || *c > b'~' {
								self.collect_state = Ended(Err(HeaderLine));
								return false;
							}

							if *c == b':' && split_idx.is_none() {
								split_idx = Some(i);
							}
						}

						let header_line = header_line.trim_ascii();
						if header_line.is_empty() {
							self.collect_state = Processing(Body);
							return true;
						}

						if split_idx.is_none() {
							self.collect_state = Ended(Err(HeaderLine));
							return false;
						}

						let split_idx = split_idx.unwrap();

						let k = header_line[..split_idx].trim_ascii();
						let v = header_line[split_idx + 1..].trim_ascii();

						self.headers.push((k.into(), v.into()));

						if k.eq_ignore_ascii_case(b"content-length") {
							if self.content_length.is_some() {
								self.collect_state = Ended(Err(RepeatedContentLengthHeader));
								return false;
							}
							match v.parse::<usize>() {
								Ok(v) => self.content_length = Some(v),
								Err(_e) => {
									self.collect_state = Ended(Err(InvalidHeader));
									return false;
								}
							}
						}

						true
					}
				}
			}
			Body => {
				return false;
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
		MessageCommon::<RequestSpecific>::new(
			RequestSpecific {
				method: None,
				url: None,
			}
		)
	}
}

impl ResponseCollector {
	pub fn new() -> Self {
		MessageCommon::<ResponseSpecific>::new(
			ResponseSpecific {
				status_code: None,
				status_desc: None,
			}
		)
	}
}
