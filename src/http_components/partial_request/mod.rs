#[cfg(test)]
mod tests;

use std::cmp::PartialEq;
use std::fmt::Debug;
use std::io::BufRead;
use std::mem::swap;
use std::str::FromStr;
use log::debug;
use crate::{HTTPAttachment, HTTPHeader, HTTPRequest, MimeType, Url};
use crate::MimeType::Multipart;

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
					"content-type" => {
						self.partially_parsed_request.mime_type =
							HTTPHeader::parse_content_type_value(&header.value);
					}
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

		match &self.partially_parsed_request.mime_type {
			Multipart(boundary) => {
				self.partially_parsed_request.attachments =
					self.extract_attachments(boundary);
			}
			_ => {}
		};

		Some(self.partially_parsed_request)
	}

	fn extract_attachments(&self, boundary: &str) -> Vec<HTTPAttachment> {
		let mut extracted_attachments = Vec::<HTTPAttachment>::new();

		let mut is_first_boundary = true;
		let magic_sequence = format!("\r\n--{boundary}\r\n");
		let magic_sequence_first_time = &magic_sequence[2..];

		let magic_sequence_eof = format!("\r\n--{boundary}--\r\n");

		if !self.partially_parsed_request.body
			.ends_with(magic_sequence_eof.as_bytes()) {
			println!("body doesn't end with magic sequence - bad request/400");
			return Vec::default();
		}

		let content = &self.partially_parsed_request
			.body[0..self.partially_parsed_request.body.len() - magic_sequence_eof.len()];

		let mut current_boundary_start_index = 0;

		for mut i in 0..content.len() {
			let is_last_byte = i + 1 == content.len();

			let current_magic_sequence =
				if is_first_boundary {
					magic_sequence_first_time.as_bytes()
				} else {
					magic_sequence.as_bytes()
				};

			let is_boundary = content[i..]
				.starts_with(current_magic_sequence);

			if !is_boundary && !is_last_byte {
				continue;
			}

			if is_first_boundary {
				is_first_boundary = false;
			}

			let current_boundary_size = i - current_boundary_start_index
				+ if is_last_byte { 1 } else { 0 };

			if current_boundary_size == 0 {
				continue
			}

			let attachment =
				Self::parse_attachment(&content[
					current_boundary_start_index..
						current_boundary_start_index + current_boundary_size]);

			extracted_attachments.push(attachment);

			i += current_magic_sequence.len();
			current_boundary_start_index = i;
		}

		extracted_attachments
	}

	fn parse_attachment(text: &[u8]) -> HTTPAttachment {
		let mut end_of_header_section = 0usize;
		let mut local_new_line_hold = false;
		for i in 0..text.len() {
			if text[i] == b'\n' {
				if local_new_line_hold {
					end_of_header_section = i + 1;
					break;
				} else {
					local_new_line_hold = true;
				}
			} else if text[i] != b'\r' {
				local_new_line_hold = false;
			}
		}

		let (headers_section, body) = text.split_at(end_of_header_section);

		let mut headers = Vec::<HTTPHeader>::new();
		let mut mime_type= MimeType::TextPlain;
		let mut name = String::new();
		let mut filename: Option<String> = None;

		let mut lines_it = headers_section.lines();
		while let Some(Ok(line)) = lines_it.next() {
			if line.is_empty() {
				break;
			}
			let header = HTTPHeader::from_str(line.as_str())
				.expect("again, braking changes if this fails, remember to change this");

			match header.name.to_lowercase().as_ref() {
				"content-type" => mime_type =
					HTTPHeader::parse_content_type_value(header.value.as_str()),
				"content-disposition" => {
					let mut splits = header.value.split(';');
					match splits.next() {
						Some(v) => {
							if v != "form-data" {
								continue;
							}
						}
						None => continue
					};

					while let Some(key_val_part) = splits.next() {
						let (key, value) = match key_val_part.find('=') {
							None => (key_val_part, ""),
							Some(index) => key_val_part.split_at(index)
						};

						let value = {
							let no_eq_sign = &value[1..];
							let no_starting_quote = if no_eq_sign.starts_with('"') {
								&no_eq_sign[1..]
							} else {
								no_eq_sign
							};
							if no_starting_quote.ends_with('"') {
								&no_starting_quote[..no_starting_quote.len()-1]
							} else {
								no_starting_quote
							}
						};

						match key.trim() {
							"name" => name = value.to_string(),
							"filename" => filename = Some(value.to_string()),
							_ => continue
						};
					}
				}
				_ => {}
			}

			headers.push(header);
		};

		HTTPAttachment {
			name,
			headers,
			mime_type,
			filename,
			data: body.to_vec(),
		}
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
