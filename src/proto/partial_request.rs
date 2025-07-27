use crate::proto::internal::message_parser::MessageParser;

pub struct HTTPPartialRequest {
	message_parser: MessageParser
}

impl Default for HTTPPartialRequest {
	fn default() -> Self {
		Self {
			message_parser: MessageParser::new_request()
		}
	}
}

impl HTTPPartialRequest {
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
		self.message_parser.push_bytes(bytes)
	}
}
