// todo: set max buffer limit

use crate::http_components::buffer_reader::BufferReader;
use crate::http_components::message::HTTPMessage;

enum ParsePart {
	FirstLineRequest,
	FirstLineResponse,

	MainHeaders,
	MainBody,

	MultipartStart,
	AttachmentHeaders,
	AttachmentBody,
	MultipartEnd,
}

enum ParseAction {
	Continue,
	ChangePart(ParsePart),
	Finish,
}

pub struct PartialMessage {

	inner_buffer: BufferReader,
	inner_partial_message: HTTPMessage,
}

trait PartialMessagePushBytes {
	fn push_bytes(&mut self, bytes: &[u8]) {

	}
}

impl PartialMessage {

	fn push_bytes(&mut self, bytes: &[u8]) {

	}
}
