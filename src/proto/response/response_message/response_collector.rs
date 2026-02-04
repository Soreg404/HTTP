use crate::consts::{StatusCode, Version};
use crate::proto::message::{CollectResult, MessageCollector, MessageCollectorAdvance};
use crate::proto::parser;
use crate::proto::parser::ParseError;
use crate::request::Request;
use crate::response::Response;

pub struct ResponseCollector {
	collect_result: CollectResult,

	version: Option<Version>,
	status_code: Option<StatusCode>,
	status_desc: Option<String>,

	message_collector: MessageCollector,

	internal_buffer: Vec<u8>,
}

impl ResponseCollector {
	pub fn new() -> Self {
		Self {
			collect_result: None,
			version: None,
			status_code: None,
			status_desc: None,
			message_collector: MessageCollector::new(),
			internal_buffer: Vec::new(),
		}
	}

	pub fn is_finished(&self) -> bool {
		self.collect_result.is_some()
	}

	pub fn into_response(self) -> Result<Response, ParseError> {
		match self.collect_result {
			None => panic!("Attempted to convert an incomplete response"),
			Some(Err(e)) => Err(e),
			Some(Ok(())) => Ok(Response {
				status_code: self.status_code.unwrap(),
				status_desc: self.status_desc.unwrap(),
				message: self.message_collector.into_message(self.version.unwrap()),
			})
		}
	}
}

impl ResponseCollector {
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
		if self.is_finished() {
			panic!("Finished");
		}
		self.internal_buffer.extend_from_slice(bytes);

		match self.message_collector.advance(
			self.internal_buffer.as_slice(),
			|s| {
				let v = parser::parse_response_first_line(s)?;


				self.version = Some(v.version);
				self.status_code = Some(v.status_code);
				self.status_desc = Some(v.status_desc);

				Ok(())
			},
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
