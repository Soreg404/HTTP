#[cfg(test)]
mod tests;

use std::cmp::PartialEq;
use std::fmt::Debug;
use std::mem::swap;
use std::str::FromStr;
use crate::{HTTPHeader, HTTPRequest, Url};
use crate::http_components::mime_types::MimeType;

#[derive(PartialEq)]
enum HTTPPart {
	FirstLine,
	RequestHeaders,
	RequestData,
	AttachmentHeaders(usize),
	AttachmentData,
}

pub struct HTTPAttachment {
	number: usize,
	mime: MimeType,
	headers: Vec<HTTPHeader>,
	data: Vec<u8>,
}

pub struct HTTPPartialRequest {
	part: HTTPPart,
	parse_ended: bool,
	internal_buffer: Vec<u8>,

	new_line_hold: bool,

	content_length: usize,

	method: String,
	url: Url,
	http_version: String,
	headers: Vec<HTTPHeader>,
	body: Vec<u8>,
}
impl Default for HTTPPartialRequest {
	fn default() -> Self {
		HTTPPartialRequest {
			part: HTTPPart::FirstLine,
			parse_ended: false,

			internal_buffer: Vec::with_capacity(0x400),

			new_line_hold: false,
			content_length: 0,

			method: String::default(),
			url: Url::default(),
			http_version: String::default(),
			headers: Vec::default(),
			body: Vec::default(),
		}
	}
}


impl HTTPPartialRequest {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn from_str(text: &str) -> Self {
		let mut s = Self::default();
		s.push_bytes(text.as_bytes());
		s
	}

	pub fn push_bytes(&mut self, buffer: &[u8]) {
		for c in buffer.iter().cloned() {
			if self.is_complete() {
				return;
			}

			self.internal_buffer.push(c);

			if c == b'\n' {
				match &self.part {
					HTTPPart::FirstLine => {
						self.parse_request_first_line()
							.expect("bad first line");
						self.part = HTTPPart::RequestHeaders;
						self.internal_buffer.clear();
					}
					HTTPPart::RequestHeaders => {
						if self.new_line_hold {
							self.part = HTTPPart::RequestData;
							self.internal_buffer.clear();
							continue;
						}

						let current_header_line = String::from_utf8_lossy(
							self.internal_buffer.as_slice()
						)
							.to_string();

						let header = HTTPHeader::from_line(&current_header_line);



						match header.name.to_lowercase().as_ref() {
							"content-length" => {
								self.content_length = match header.value.parse::<usize>() {
									Ok(v) => v,
									_ => 0
								}
							}
							"content-type" => {}
							_ => {}
						};

						self.parsed_request.headers.push(header);
						self.internal_buffer.clear();
					}
					_ => {}
				}
				self.new_line_hold = true;
			} else if c != b'\r' {
				self.new_line_hold = false;
			}

			if self.part == RequestData
				&& self.internal_buffer.len() >= self.content_length {
				swap(&mut self.parsed_request.body, &mut self.internal_buffer);
				self.parse_ended = true;
			}
		}
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
			self.method = line_str.get(0).unwrap().to_owned();
			self.url = Url::from_str(line_str.get(1).unwrap())
				.unwrap_or_default();
			self.http_version = line_str.get(2).unwrap().to_owned();
			Ok(())
		}
	}

	pub fn is_complete(&self) -> bool {
		self.parse_ended
	}

	pub fn get_complete_request(mut self) -> Option<HTTPRequest> {
		if !self.parse_ended {
			return None;
		}
		Some(
			HTTPRequest {
				method: self.method,
				url: self.url,
				http_version: self.http_version,
				headers: self.headers,
				body: self.body,
			}
		)
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
