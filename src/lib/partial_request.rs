use std::fmt::Debug;
use crate::lib::{HTTPHeader, HTTPRequest};

pub struct HTTPPartialRequest {
	part_counter: u8,
	is_complete: bool,
	internal_buffer: Vec<u8>,

	new_line_hold: bool,

	content_length: usize,

	parsed_request: HTTPRequest,
}
impl Default for HTTPPartialRequest {
	fn default() -> Self {
		HTTPPartialRequest {
			part_counter: 0,
			is_complete: false,
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

	fn parse_request_first_line(&mut self) -> Result<(), &'static str> {
		let line_str = String::from_utf8_lossy(
			self.internal_buffer.as_slice())
			.to_string();
		let line_trim = line_str.trim();


		let mut subpart = 0;
		let mut whitespace_hold = false;
		let mut buffer = Vec::<char>::with_capacity(0x400);
		let mut it = line_trim.chars().peekable();
		while let Some(c) = it.next() {
			if c.is_whitespace() || it.peek() == None {
				if it.peek() == None {
					buffer.push(c);
				}
				if !whitespace_hold {
					whitespace_hold = true;

					match subpart {
						0 => self.parsed_request.method = buffer.iter().collect(),
						1 => {
							let uri: String = buffer.iter().collect();
							let mut uri_it = uri.split('?');
							self.parsed_request.path = uri_it.next().unwrap().to_string();
							self.parsed_request.query = match (uri_it.next()) {
								Some(v) => String::from(v),
								None => String::new(),
							};
						}

						2 => self.parsed_request.http_version = buffer.iter().collect(),
						_ => return Err("bad format")
					}
					buffer.clear();
					subpart += 1;
				}
			} else {
				whitespace_hold = false;
				buffer.push(c);
			}
		}

		Ok(())
	}

	pub fn push_bytes(&mut self, buffer: &[u8]) {
		for c in buffer.iter().cloned() {
			if self.is_complete {
				return;
			}

			if c == b'\r' {
				continue;
			}

			if c != b'\n' {
				self.internal_buffer.push(c);
			}

			if c == b'\n' {
				match self.part_counter {
					0 => {
						self.parse_request_first_line()
							.expect("bad first line");
						self.part_counter = 1;
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
							self.part_counter = 2;
						}
					}
					_ => {}
				}
				self.new_line_hold = true;
			} else {
				self.new_line_hold = false;
			}

			if self.part_counter == 2
				&& !self.is_complete
				&& self.internal_buffer.len() >= self.content_length {
				self.parsed_request.body.clone_from(&self.internal_buffer);
				self.internal_buffer.clear();
				self.is_complete = true;
			}
		}
	}

	pub fn is_complete(&self) -> bool {
		self.is_complete
	}

	pub fn get_complete_request(&self) -> Option<&HTTPRequest> {
		if !self.is_complete {
			return None;
		}
		Some(&self.parsed_request)
	}
}
impl Debug for HTTPPartialRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "HTTP partial request (complete={})",
				 if self.is_complete { "true" } else { "false" })?;
		writeln!(f, "{:?}", self.parsed_request)?;
		Ok(())
	}
}
// impl Display for HTTPPartialRequest {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
// 		writeln!(f, "{self:?}")
// 	}
// }
