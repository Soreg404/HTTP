use super::{buffer_reader::BufferReader, message::HTTPMessage, parser};
use crate::{
	Url,
	HTTPParseError,
};

enum ParseState {
	FirstLineRequest,
	FirstLineResponse,

	MainHeaders,
	MainBody,

	MultipartStart,
	AttachmentHeaders,
	AttachmentBody,
	MultipartEnd,
}

// struct ParseValues {
// 	content_length: usize
// }

struct MessageParser {
	internal_buffer: BufferReader,

	state: ParseState,
	result: Option<Result<(), HTTPParseError>>,

	request_method: String,
	request_url_target: Url,

	response_status_code: u16,
	response_status_text: String,

	incomplete_message: HTTPMessage,
}

impl MessageParser {
	fn new_request() -> Self {
		Self {
			state: ParseState::FirstLineRequest,
			..Default::default()
		}
	}

	fn new_response() -> Self {
		Self {
			state: ParseState::FirstLineResponse,
			..Default::default()
		}
	}
}

impl MessageParser {
	fn push_bytes(&mut self, data: &[u8]) -> usize {
		self.internal_buffer.append(data);
		let starting_head = self.internal_buffer.get_head_idx();

		self.process_buffer();

		let processed_bytes = self.internal_buffer.get_head_idx() - starting_head;
		processed_bytes
	}

	fn process_buffer(&mut self) {
		while self.result.is_none()
			&& self.internal_buffer.advance() {
			self.match_state();
		}
	}

	fn match_state(&mut self) {
		match self.state {
			ParseState::FirstLineRequest => {
				match self.internal_buffer.read_line() {
					None => return,
					Some(line) => {
						let flr = match parser::get_first_line_request(line) {
							Ok(flr) => flr,
							Err(e) => {
								self.result = Some(Err(e));
								return;
							}
						};

						self.request_method = flr.method;
						self.request_url_target = flr.url;
						self.incomplete_message.http_version = flr.version;

						self.state = ParseState::MainHeaders;
					}
				};
			}
			ParseState::FirstLineResponse => {
				match self.internal_buffer.read_line() {
					None => return,
					Some(line) => {
						let flr = match parser::get_first_line_response(line) {
							Ok(flr) => flr,
							Err(e) => {
								self.result = Some(Err(e));
								return;
							}
						};

						self.incomplete_message.http_version = flr.version;
						self.response_status_code = flr.status_code;
						self.response_status_text = flr.status_text;

						self.state = ParseState::MainHeaders;
					}
				};
			}

			ParseState::MainHeaders => {
				match self.internal_buffer.read_line() {
					None => return,
					Some(line) => {
						if line.is_empty() {
							todo!("done headers")
						}


					}
				}
			}
			ParseState::MainBody => {}

			ParseState::MultipartStart => {}
			ParseState::AttachmentHeaders => {}
			ParseState::AttachmentBody => {}
			ParseState::MultipartEnd => {}
		}
	}
}
