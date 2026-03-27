use crate::consts::{Method, StatusCode, Version};
use crate::proto::message_common::CollectMessageState::Ended;
use crate::proto::parser::ParseError;
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

	pub working_buffer: Vec<u8>,
	pub working_buffer_reader: StateBufferReader,

	collect_state: CollectMessageState,
}

trait MessageSpecific {
	fn collect_first_line(
		&mut self,
		line: &[u8],
	) -> Result<Version, ParseError>;
}

enum ProcessLoopOutcome {
	NotEnoughBytes,
	Continue
}

impl<T> MessageCommon<T>
where
	T: MessageSpecific,
{
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
		if let Ended(_) = self.collect_state {
			return 0;
		}

		{
			self.working_buffer.extend_from_slice(bytes);
		}

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
						continue
					} else {
						break
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
						// todo: check line validity

						let header_line = header_line.trim_ascii();

						if header_line.is_empty() {
							self.collect_state = Processing(Body);
							return true;
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
		MessageCommon::<RequestSpecific> {
			message_specific: RequestSpecific {
				method: None,
				url: None,
			},
			version: None,
			working_buffer: Vec::new(),
			working_buffer_reader: StateBufferReader::new(),
			collect_state: Processing(FirstLine),
		}
	}
}

impl ResponseCollector {
	pub fn new() -> Self {
		MessageCommon::<ResponseSpecific> {
			message_specific: ResponseSpecific {
				status_code: None,
				status_desc: None,
			},
			version: None,
			working_buffer: Vec::new(),
			working_buffer_reader: StateBufferReader::new(),
			collect_state: Processing(FirstLine),
		}
	}
}
