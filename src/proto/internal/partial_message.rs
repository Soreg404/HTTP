use crate::proto::attachment::HTTPAttachment;
use super::{buffer_reader::BufferReader, message::HTTPMessage, parser};
use crate::proto::header::HTTPHeaderEnum;
use crate::proto::internal::endline::{strip_last_endl};
use crate::proto::mime_type::MimeType;
use crate::proto::parse_error::HTTPParseError::MalformedMessage;
use crate::proto::parse_error::{HTTPParseError, MalformedMessageKind};
use crate::proto::request::HTTPRequest;
use crate::proto::response::HTTPResponse;
use crate::proto::Url;

#[derive(Debug, Eq, PartialEq)]
enum MessageIs {
	Request,
	Response,
}

enum ParseState {
	FirstLineRequest,
	FirstLineResponse,

	MainHeaders,
	MainBody,

	MultipartStart,
	AttachmentHeaders,
	AttachmentBody,
	MultipartEnd,
	MainBodyStart,
}

pub struct HTTPPartialMessage {
	message_is: MessageIs,

	internal_buffer: BufferReader,

	state: ParseState,
	result: Option<Result<(), HTTPParseError>>,

	expect_content_length: Option<usize>,
	expect_mime_type: Option<MimeType>,

	request_method: String,
	request_url_target: Url,

	response_status_code: u16,
	response_status_text: String,

	body_start_index: usize,
	incomplete_message: HTTPMessage,
}

impl HTTPPartialMessage {
	pub fn new_request() -> Self {
		Self {
			message_is: MessageIs::Request,
			state: ParseState::FirstLineRequest,

			internal_buffer: Default::default(),
			result: None,
			expect_content_length: None,
			expect_mime_type: None,
			request_method: "".to_string(),
			request_url_target: Default::default(),
			response_status_code: 0,
			response_status_text: "".to_string(),
			body_start_index: 0,
			incomplete_message: Default::default(),
		}
	}

	pub fn new_response() -> Self {
		Self {
			message_is: MessageIs::Response,
			state: ParseState::FirstLineResponse,

			internal_buffer: Default::default(),
			result: None,
			expect_content_length: None,
			expect_mime_type: None,
			request_method: "".to_string(),
			request_url_target: Default::default(),
			response_status_code: 0,
			response_status_text: "".to_string(),
			body_start_index: 0,
			incomplete_message: Default::default(),
		}
	}
}

impl HTTPPartialMessage {
	pub fn push_bytes(&mut self, data: &[u8]) -> usize {
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
				println!("first line request - state continue");
				match self.internal_buffer.read_line() {
					None => return,
					Some(line) => {
						println!("first line request - line {:?}", String::from_utf8_lossy(line));

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
				println!("first line response - state continue");
				match self.internal_buffer.read_line() {
					None => return,
					Some(line) => {
						println!("first line response - line {:?}", String::from_utf8_lossy(line));
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
				println!("main headers - continue");
				match self.internal_buffer.read_line() {
					None => return,
					Some(line) => {
						println!("main headers - line {line:?}");
						if line.is_empty() {
							self.state = ParseState::MainBodyStart;
							return;
						}

						let header = match parser::header_from_line_bytes(line) {
							Ok(h) => h,
							Err(e) => {
								self.result = Some(Err(e));
								return;
							}
						};

						self.incomplete_message.headers.push(header);
						let header = self.incomplete_message
							.headers.last().unwrap();

						match HTTPHeaderEnum::from_header(&header) {
							Err(e) => {
								// todo: skip or bail? some errors better than others?
								self.result = Some(Err(e));
								return;
							}
							Ok(v) => {
								match v {
									HTTPHeaderEnum::ContentLength(len) => {
										self.expect_content_length = Some(len)
									}
									HTTPHeaderEnum::ContentType(mime) => {
										self.expect_mime_type = Some(mime)
									}
									_ => {}
								}
							}
						}
					}
				}
			}
			ParseState::MainBodyStart => {
				println!("main body start - continue");
				// match self.expect_content_length {
				// 	None | Some(ref len) if *len == 0 => {
				// 		self.result = Some(Ok(()));
				// 		return;
				// 	}
				// 	_ => {}
				// };

				self.body_start_index = self.internal_buffer.get_head_idx();

				self.state = match self.expect_mime_type {
					Some(MimeType::Multipart(_)) => ParseState::MultipartStart,
					_ => ParseState::MainBody,
				}
			}

			ParseState::MainBody => {
				println!("main body - continue");

				let len = self.expect_content_length
					.unwrap_or(0);

				match self.internal_buffer.read_exact(len) {
					None => return,
					Some(body) => {
						println!("main body - got contents - finish");
						self.incomplete_message.body = body.to_vec();
					}
				}

				self.result = Some(Ok(()));
			}

			ParseState::MultipartStart => {
				println!("multipart start - continue");

				let line = match self.internal_buffer.read_line() {
					None => return,
					Some(line) => line
				};
				println!("multipart start - line {line:?}");

				let MimeType::Multipart(boundary) =
					self.expect_mime_type
						.as_ref()
						.expect("multipart mime type should be set here, in MultipartStart") else { todo!() };

				let boundary_with_prefix = format!("--{boundary}");

				if line != boundary_with_prefix.as_bytes() {
					self.result = Some(Err(
						MalformedMessage(
							MalformedMessageKind::MultipartFirstLineBoundary
						)
					));
					return;
				}

				self.state = ParseState::AttachmentHeaders;
			}
			ParseState::AttachmentHeaders => {
				println!("attachment headers - continue");

				match self.internal_buffer.read_line() {
					None => return,
					Some(line) => {
						println!("attachment headers - line {line:?}");

						if line.is_empty() {
							self.state = ParseState::AttachmentBody
						}

						let header = match parser::header_from_line_bytes(line) {
							Ok(h) => h,
							Err(e) => {
								self.result = Some(Err(e));
								return;
							}
						};

						let mut last_attachment = self.get_last_attachment();

						last_attachment.headers.push(header);
						let header = last_attachment
							.headers.last().unwrap();

						match HTTPHeaderEnum::from_header(&header) {
							Err(e) => {
								// todo: skip or bail? some errors better than others?
								self.result = Some(Err(e));
								return;
							}
							Ok(v) => {
								match v {
									HTTPHeaderEnum::ContentType(mime) => {
										last_attachment.mime_type = mime
									}
									HTTPHeaderEnum::ContentDispositionFormData(data) => {
										last_attachment.name = data.name;
										last_attachment.filename = data.filename;
									}
									_ => {}
								}
							}
						}
					}
				}
			}
			ParseState::AttachmentBody => {
				let MimeType::Multipart(boundary) =
					self.expect_mime_type
						.as_ref()
						.expect("multipart mime type should be set here, in AttachmentBody") else { todo!() };

				let boundary_with_prefix = format!("--{boundary}");

				match self.internal_buffer.read_until(boundary_with_prefix.as_bytes()) {
					None => return,
					Some(body) => {
						let body = strip_last_endl(body);
						self.get_last_attachment().body = body.to_vec();
					}
				}

				self.state = ParseState::MultipartEnd;
			}
			ParseState::MultipartEnd => {
				match self.internal_buffer.read_line() {
					None => return,
					Some(line) => {
						if line.is_empty() {
							self.state = ParseState::AttachmentHeaders
						} else if line == b"--" {
							self.result = Some(Ok(()))
						} else {
							self.result = Some(Err(MalformedMessage(
								MalformedMessageKind::MultipartEndsWithInvalidBytes
							)))
						}
					}
				}
			}
		}
	}

	fn get_last_attachment(&mut self) -> &mut HTTPAttachment {
		let mut ats = &mut self.incomplete_message.attachments;
		if ats.is_empty() {
			ats.push(HTTPAttachment::default())
		}
		ats.last_mut().unwrap()
	}

	pub fn is_complete(&self) -> bool {
		self.result.is_some()
	}

	pub fn into_request(self) -> HTTPRequest {
		assert_eq!(self.message_is, MessageIs::Request);
		HTTPRequest {
			method: self.request_method,
			url: self.request_url_target,
			message: self.incomplete_message,
		}
	}

	pub fn into_response(self) -> HTTPResponse {
		assert_eq!(self.message_is, MessageIs::Response);
		HTTPResponse {
			status_code: self.response_status_code,
			status_text: self.response_status_text,
			message: self.incomplete_message,
		}
	}
}
