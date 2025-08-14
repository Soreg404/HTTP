use crate::proto::internal::partial_message::HTTPPartialMessage;
use crate::{HTTPParseError, HTTPResponse};

#[derive(Default)]
pub struct HTTPPartialResponse {
	// todo: make it references to partial_message's internal_buffer
	status_code: u16,
	status_text: String,

	partial_message: HTTPPartialMessage,
}

impl HTTPPartialResponse {
	pub fn push_bytes(&mut self, bytes: &[u8]) {
		self.partial_message.push_bytes(bytes);

		if self.partial_message.is_first_line() {
			let first_line = match self.partial_message.take_first_line_response() {
				None => return,
				Some(v) => v
			};

			self.status_code = first_line.status_code;
			self.status_text = first_line.status_text;
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

impl TryInto<HTTPResponse> for HTTPPartialResponse {
	type Error = HTTPParseError;
	fn try_into(self) -> Result<HTTPResponse, Self::Error> {
		let message = self.partial_message.try_into()?;

		Ok(
			HTTPResponse {
				status_code: self.status_code,
				status_text: self.status_text,
				message,
			}
		)
	}
}

impl HTTPPartialResponse {
	pub fn into_response(self) -> Result<HTTPResponse, HTTPParseError> {
		self.try_into()
	}
}
