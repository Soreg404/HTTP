use crate::HTTPResponse;
use crate::proto::internal::partial_message::HTTPPartialMessage;

pub struct HTTPPartialResponse {
	message_parser: HTTPPartialMessage
}

impl Default for HTTPPartialResponse {
	fn default() -> Self {
		Self {
			message_parser: HTTPPartialMessage::new_response()
		}
	}
}

impl HTTPPartialResponse {
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
		self.message_parser.push_bytes(bytes)
	}

	pub fn is_complete(&self) -> bool {
		self.message_parser.is_complete()
	}

	pub fn into_response(self) -> HTTPResponse {
		self.message_parser.into_response()
	}
}
