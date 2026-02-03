use crate::consts::{Method, Version};
use crate::proto::buffer_reader::{DelayedConsumeResult, DelayedStateBuffer};
use crate::proto::message::{ CollectResult, CollectorState, MessageCollector, MessageCollectorAdvance};
use crate::proto::parser;
use crate::proto::parser::ParseError;
use crate::request::Request;

pub struct RequestCollector {
	collect_result: CollectResult,

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
				message: self.message_collector.into_message(self.version.unwrap()),
			})
		}
	}
}

impl RequestCollector {
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
		if self.is_finished() {
			panic!("Attempted to push bytes on a finished collector");
		}
		self.internal_buffer.extend_from_slice(bytes);

		match self.message_collector.advance(
			self.internal_buffer.as_slice(),
			|s| {
				let v = parser::parse_request_first_line(s)?;

				self.method = Some(v.method);
				self.url = Some(String::from_utf8_lossy(
					v.url_slice.as_slice()).to_string());
				self.version = Some(v.version);

				Ok(())
			}
		) {
			MessageCollectorAdvance::NeedMoreBytes => bytes.len(),
			MessageCollectorAdvance::Finished { remaining_bytes } => {
				self.collect_result = Some(Ok(()));
				dbg!(bytes.len(), remaining_bytes);
				bytes.len() - remaining_bytes
			}
			MessageCollectorAdvance::Error(e) => {
				self.collect_result = Some(Err(e));
				0
			}
		}

	}
}
