use std::fmt::Debug;
use crate::{HTTPHeader, HTTPRequest, Url};
use crate::http_components::mime_types::MimeType;
use crate::HTTPPart::{FirstLine, RequestHeaders};

pub enum HTTPPart {
	FirstLine,
	RequestHeaders,
	RequestData,
	AttachmentHeaders(usize),
	AttachmentData
}

pub struct HTTPAttachment {
	number: usize,
	mime: MimeType,
	headers: Vec<HTTPHeader>,
	data: Vec<u8>
}

pub struct HTTPPartialRequest {
	part: HTTPPart,
	parse_ended: bool,
	internal_buffer: Vec<u8>,

	new_line_hold: bool,

	content_length: usize,

	parsed_request: HTTPRequest,
}
impl Default for HTTPPartialRequest {
	fn default() -> Self {
		HTTPPartialRequest {
			part: FirstLine,
			parse_ended: false,
			internal_buffer: Vec::with_capacity(0x400),

			new_line_hold: false,
			content_length: 0,

			parsed_request: HTTPRequest::default(),
		}
	}
}
impl HTTPPartialRequest {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn from_str(text: impl AsRef<str>) -> Self {
		let mut s = Self::default();
		s.push_bytes(text.as_ref().as_bytes());
		s
	}

	fn parse_request_first_line(&mut self) -> Result<(), ()> {
		let line_str = String::from_utf8_lossy(
			self.internal_buffer.as_slice()
		)
			.split_whitespace()
			.map(|s| s.to_owned())
			.collect::<Vec<String>>();

		if line_str.len() != 3 {
			Err(())
		} else {
			self.parsed_request.method = line_str.get(0).unwrap().to_owned();
			self.parsed_request.url = Url::from_request_str(line_str.get(1).unwrap());
			self.parsed_request.http_version = line_str.get(2).unwrap().to_owned();
			Ok(())
		}
	}

	pub fn push_bytes(&mut self, buffer: &[u8]) {
		for c in buffer.iter().cloned() {
			if self.parse_ended {
				return;
			}

			self.internal_buffer.push(c);

			if c == b'\n' {
				match self.part {
					HTTPPart::FirstLine => {
						self.parse_request_first_line()
							.expect("bad first line");
						self.part = RequestHeaders;
						self.internal_buffer.clear();
					}
					1 => {
						if !self.new_line_hold {
							let current_header_line = String::from_utf8_lossy(self.internal_buffer.as_slice())
								.to_string();
							let mut header_parts_it = current_header_line.split(':');
							let mut header_name = header_parts_it.next()
								.unwrap().trim().to_string();
							let mut header_val = match (header_parts_it.next()) {
								Some(v) => String::from(v).trim().to_string(),
								None => String::new(),
							};
							if header_name.to_lowercase() == "content-length" {
								self.content_length = match header_val.parse::<usize>() {
									Ok(v) => v,
									_ => 0
								};
							}

							self.parsed_request.headers.push(HTTPHeader {
								name: header_name,
								value: header_val,
							});
							self.internal_buffer.clear();
						} else {
							self.part = 2;
						}
					}
					_ => {
						self.internal_buffer.push(c);
					}
				}
				self.new_line_hold = true;
			} else if c != b'\r' {
				self.new_line_hold = false;
			}

			if self.part == 2
				&& !self.parse_ended
				&& self.internal_buffer.len() >= self.content_length {
				self.parsed_request.body.clone_from(&self.internal_buffer);
				self.internal_buffer.clear();
				self.parse_ended = true;
			}
		}
	}

	pub fn is_complete(&self) -> bool {
		self.parse_ended
	}

	pub fn get_complete_request(&self) -> Option<&HTTPRequest> {
		if !self.parse_ended {
			return None;
		}
		Some(&self.parsed_request)
	}
}
impl Debug for HTTPPartialRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "HTTP partial request (complete={})",
				 if self.parse_ended { "true" } else { "false" })?;
		writeln!(f, "{:?}", self.parsed_request)?;
		Ok(())
	}
}
// impl Display for HTTPPartialRequest {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
// 		writeln!(f, "{self:?}")
// 	}
// }
