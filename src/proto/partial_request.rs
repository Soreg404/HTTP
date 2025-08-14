use crate::{HTTPParseError, HTTPPartialResponse, HTTPRequest, HTTPResponse};
use crate::proto::internal::message::HTTPMessage;
use crate::proto::internal::parser;
use crate::proto::internal::partial_message::HTTPPartialMessage;

#[derive(Default)]
pub struct HTTPPartialRequest {
	// todo: make it references to partial_message's internal_buffer
	method: String,
	target: String,

	partial_message: HTTPPartialMessage
}

impl HTTPPartialRequest {
	pub fn push_bytes(&mut self, bytes: &[u8]) {
		self.partial_message.push_bytes(bytes);

		if self.partial_message.is_first_line() {
			let first_line = match self.partial_message.take_first_line_request() {
				None => return,
				Some(v) => v
			};

			self.method = first_line.method;
			self.target = first_line.target;
		}

		self.partial_message.advance();
	}

	pub fn advance(&mut self) {
		self.partial_message.advance()
	}

	pub fn is_finished(&self) -> bool {
		self.partial_message.is_finished()
	}

	pub fn signal_connection_closed(&mut self) {
		self.partial_message.signal_connection_closed()
	}
}

impl TryInto<HTTPRequest> for HTTPPartialRequest {
	type Error = HTTPParseError;
	fn try_into(self) -> Result<HTTPRequest, Self::Error> {
		let message = self.partial_message.try_into()?;

		Ok(
			HTTPRequest {
				method: self.method,
				target: self.target,
				message,
			}
		)
	}
}

impl HTTPPartialRequest {
	pub fn into_request(self) -> Result<HTTPRequest, HTTPParseError> {
		self.try_into()
	}
}
