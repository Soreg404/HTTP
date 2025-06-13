#[cfg(test)]
mod tests;

use std::cmp::PartialEq;
use std::fmt::Debug;
use std::mem::swap;
use std::str::FromStr;
use log::debug;
use crate::{HTTPHeader, HTTPRequest, Url};

#[derive(PartialEq)]
enum HTTPPart {
	FirstLine,
	RequestHeaders,
	RequestData,
	AttachmentHeaders(usize),
	AttachmentData,
}

pub struct HTTPPartialRequest {
	part: HTTPPart,
	parse_ended: bool,
	internal_buffer: Vec<u8>,

	new_line_hold: bool,

	content_length: Option<usize>,

	partially_parsed_request: HTTPRequest,
}
impl Default for HTTPPartialRequest {
	fn default() -> Self {
		HTTPPartialRequest {
			part: HTTPPart::FirstLine,
			parse_ended: false,

			internal_buffer: Vec::with_capacity(0x400),

			new_line_hold: false,
			content_length: None,

			partially_parsed_request: HTTPRequest::default(),
		}
	}
}

impl HTTPPartialRequest {
	pub fn push_bytes(&mut self, buffer: &[u8]) {
		for c in buffer.iter().cloned() {
			if self.is_complete() {
				return;
			}

			self.internal_buffer.push(c);

			if c == b'\n' {
				self.on_new_line();
				self.new_line_hold = true;
			} else if c != b'\r' {
				self.new_line_hold = false;
			}

			if self.part == HTTPPart::RequestData {
				if self.internal_buffer.len() >= self.content_length
					// todo: temporary, error handling of wrong or duplicate content-length
					.clone().unwrap_or(0) {
					swap(&mut self.partially_parsed_request.body, &mut self.internal_buffer);

					// todo: temporary, attachments wip
					self.parse_ended = true;
				}
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
			self.partially_parsed_request.method = line_str.get(0).unwrap().to_owned();
			self.partially_parsed_request.url = Url::from_str(line_str.get(1).unwrap())
				.unwrap_or_default();
			self.partially_parsed_request.http_version = line_str.get(2).unwrap().to_owned();
			Ok(())
		}
	}

	fn on_new_line(&mut self) {
		match &self.part {
			HTTPPart::FirstLine => {
				if self.parse_request_first_line().is_err() {
					self.parse_ended = true;
					// todo: error handling
					debug!("bad first line, todo: error handling");
					return;
				}
				self.part = HTTPPart::RequestHeaders;
				self.internal_buffer.clear();
			}
			HTTPPart::RequestHeaders => {
				if self.new_line_hold {
					self.part = HTTPPart::RequestData;
					self.internal_buffer.clear();
					return;
				}

				let current_header_line = String::from_utf8_lossy(
					self.internal_buffer.as_slice()
				)
					.to_string();
				self.internal_buffer.clear();

				let header = HTTPHeader::from_str(&current_header_line)
					.expect("some breaking changes with HTTPHeader happened,\
							 remember to fix here");

				match header.name.to_lowercase().as_ref() {
					"content-length" => {
						self.content_length = match header.value.parse::<usize>() {
							Ok(v) => Some(v),
							_ => Some(0)
						}
					}
					"content-type" => {}
					_ => {}
				};

				self.partially_parsed_request.headers.push(header);
			}
			_ => {}
		}
	}

	pub fn is_complete(&self) -> bool {
		self.parse_ended
	}

	pub fn get_complete_request(mut self) -> Option<HTTPRequest> {
		if !self.parse_ended {
			return None;
		}
		Some(self.partially_parsed_request)
	}
}

impl FromStr for HTTPPartialRequest {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut ret = Self::default();
		ret.push_bytes(s.as_bytes());
		Ok(ret)
	}
}

impl Debug for HTTPPartialRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "HTTP partial request (complete={})",
				 if self.parse_ended { "true" } else { "false" })?;
		writeln!(f, "{:?}", &self.partially_parsed_request)?;
		Ok(())
	}
}
// impl Display for HTTPPartialRequest {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
// 		writeln!(f, "{self:?}")
// 	}
// }
