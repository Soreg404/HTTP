use crate::proto::internal::partial_message::HTTPPartialMessage;

pub struct HTTPPartialRequest {
	message_parser: HTTPPartialMessage
}

impl Default for HTTPPartialRequest {
	fn default() -> Self {
		Self {
			message_parser: HTTPPartialMessage::new_request()
		}
	}
}

impl HTTPPartialRequest {
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
		self.message_parser.push_bytes(bytes)
	}
}
