use crate::consts::{Method, Version};
use crate::proto::buffer_reader::{DelayedConsumeResult, DelayedStateBuffer};
use crate::proto::message::{Advance, CollectorState, MessageCollector};
use crate::proto::parser;
use crate::proto::parser::ParseError;
use crate::request::Request;

pub struct RequestCollector {
	collect_result: Option<Result<(), ParseError>>,

	is_first_line: bool,
	first_line_len: usize,

	method: Option<Method>,
	url: Option<String>,
	version: Option<Version>,

	message_collector: MessageCollector,

	internal_buffer: Vec<u8>,
}

impl RequestCollector {
	pub fn new() -> Self {
		Self {
			collect_result: None,

			is_first_line: true,
			first_line_len: 0,

			method: None,
			url: None,
			version: None,

			message_collector: MessageCollector::new(),
			internal_buffer: Vec::new(),
		}
	}

	pub fn is_finished(&self) -> bool {
		self.collect_result.is_some()
	}

	pub fn into_request(self) -> Result<Request, ParseError> {
		match self.collect_result {
			None => panic!("Attempted to convert an incomplete request"),
			Some(Err(e)) => Err(e),
			Some(Ok(())) => Ok(Request {
				method: self.method.unwrap(),
				url: self.url.unwrap(),
				version: self.version.unwrap(),
				message: self.message_collector.into_message(),
			})
		}
	}
}

impl RequestCollector {
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
		self.internal_buffer.extend_from_slice(bytes);

		let mut pushed_bytes_consumed = 0;

		if self.is_first_line {
			match DelayedStateBuffer::new()
				.take_line(&self.internal_buffer) {
				DelayedConsumeResult::NotEnoughBytes => return bytes.len(),
				DelayedConsumeResult::Finished {
					consumed,
					slice,
					..
				} => {
					self.is_first_line = false;

					match parser::parse_request_first_line(slice) {
						Ok(v) => {
							self.method = Some(v.method);
							self.url = Some(String::from_utf8_lossy(
								v.url_slice.as_slice()).to_string());
							self.version = Some(v.version);
						}
						Err(e) => {
							self.collect_result = Some(Err(e));
							return 0;
						}
					}

					self.first_line_len = consumed;

					if self.internal_buffer.len() == consumed {
						return bytes.len();
					} else {
						pushed_bytes_consumed += bytes.len()
							- (self.internal_buffer.len() - self.first_line_len);
					}
				}
			}
		}

		dbg!(bytes.len());
		dbg!(self.first_line_len);
		dbg!(pushed_bytes_consumed);

		match self.message_collector.advance(
			&self.internal_buffer[self.first_line_len..]
		) {
			Advance { consumed, collector_state } => {
				pushed_bytes_consumed += consumed;

				if let CollectorState::Finished(r) = collector_state {
					self.collect_result = Some(r);
				}
			}
		};

		dbg!(pushed_bytes_consumed);

		pushed_bytes_consumed
	}
}
