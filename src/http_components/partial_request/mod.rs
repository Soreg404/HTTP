#[cfg(test)]
mod tests;

use std::cmp::PartialEq;
use std::fmt::Debug;
use std::io::BufRead;
use std::mem::swap;
use std::str::FromStr;
use std::string::ParseError;
use log::debug;
use HTTPParseError::MalformedMessage;
use ParseRequestPart::{AttachmentBody, AttachmentHeaders, FirstLine, RequestBody, MultipartBody, RequestHeaders};
use crate::{HTTPAttachment, HTTPHeader, HTTPRequest, MimeType, Url};
use crate::http_components::buffer_reader::BufferReader;
use crate::http_components::parse_error::HTTPParseError;
use crate::http_components::parse_error::HTTPParseError::{IllegalByte, IncompleteRequest};
use crate::http_components::parse_error::MalformedMessageKind::Other;
use crate::http_components::{endline, parser, validator};
use crate::http_components::endline::{check_ends_with_new_line, strip_last_endl};
use crate::http_components::parser::ContentDispositionHeaderValuesPartial;
use crate::http_components::partial_message::PartialMessage;
use crate::http_components::partial_request::MatchPartAction::{ChangePart, Continue, Finish};
use crate::http_components::partial_request::ParseRequestPart::{MultipartCheckEnd, MultipartCheckStart};
use crate::MimeType::Multipart;

#[derive(PartialEq)]
enum ParseRequestPart {
	FirstLine,
	RequestHeaders,
	RequestBody,
	MultipartCheckStart,
	MultipartBody,
	AttachmentHeaders,
	AttachmentBody,
	MultipartCheckEnd,
}


pub struct HTTPPartialRequest {
	inner_buffer: BufferReader,

	parse_ended: bool,
	parse_error: Option<HTTPParseError>,

	method: String,
	url: Url,

	partial_message: PartialMessage,
}

impl Default for HTTPPartialRequest {
	fn default() -> Self {
		HTTPPartialRequest {
			..Self::default()
		}
	}
}


impl HTTPPartialRequest {
	pub fn push_bytes(&mut self, data: &[u8]) -> usize {
		if self.parse_ended {
			todo!()
		}

		self.inner_buffer.append(data);
		let starting_head = self.inner_buffer.get_head_idx();

		self.process_buffer();

		let processed_bytes = self.inner_buffer.get_head_idx() - starting_head;

		// todo: check if body too long

		processed_bytes
	}


	fn a() {
		while !self.parse_ended
			&& self.inner_buffer.advance() {
			match self.match_part() {
				Ok(Continue) => continue,
				Ok(ChangePart(chang_part)) => self.part = chang_part,
				Ok(Finish) => {
					self.parse_ended = true;
					break;
				}
				Err(e) => {
					self.parse_ended = true;
					self.parse_error = Some(e);
					break;
				}
			};
		}
	}

	fn match_part(&mut self) -> Result<MatchPartAction, HTTPParseError> {
		match self.part {
			FirstLine => {
				let first_line_bytes = match self.inner_buffer.read_line() {
					None => return Ok(Continue),
					Some(line) => line,
				};
				let first_line = parser::parse_request_first_line(first_line_bytes)?;
				self.partially_parsed_request.method = first_line.method;
				self.partially_parsed_request.url = first_line.url;
				self.partially_parsed_request.message.http_version = first_line.version;

				Ok(ChangePart(RequestHeaders))
			}
			RequestHeaders => {
				let line_bytes = match self.inner_buffer.read_line() {
					None => return Ok(Continue),
					Some(line) => line,
				};

				// todo: error checking, check invalid bytes

				if line_bytes.is_empty() {
					if self.parse_settings.content_length.unwrap_or(0) == 0 {
						println!("request headers, empty line - finished with body length 0");
						return Ok(Finish);
					}

					self.content_start_idx = Some(self.inner_buffer.get_head_idx());

					return Ok(ChangePart(match &self.partially_parsed_request.message.mime_type {
						Multipart => MultipartCheckStart,
						_ => RequestBody
					}));
				}

				let header = parser::header_from_line(line_bytes)?;

				match header.name.to_lowercase().as_ref() {
					"content-length" => {
						self.parse_settings.content_length =
							match header.value.parse::<usize>() {
								Ok(v) => Some(v),
								_ => Some(0)
							}
					}
					"content-type" => {
						let (mime, boundary) =
							HTTPHeader::parse_content_type_value(&header.value);
						self.partially_parsed_request.message.mime_type = mime;
						self.parse_settings.multipart_boundary = boundary;
					}
					_ => {}
				};

				self.partially_parsed_request.message.headers.push(header);

				Ok(Continue)
			}
			RequestBody => {
				let length = self.parse_settings.content_length
					.expect("content_length should be already set");

				match self.inner_buffer.read_exact(length) {
					None => Ok(Continue),
					Some(body) => {
						self.partially_parsed_request.body = body.to_vec();
						Ok(Finish)
					}
				}
			}

			MultipartCheckStart => {
				let boundary = self.parse_settings
					.multipart_boundary
					.as_ref()
					.expect("multipart_boundary should be already set");

				let first_line = match self.inner_buffer.read_line() {
					None => return Ok(Continue),
					Some(l) => l
				};

				println!("multipart check start, line={:?}", String::from_utf8_lossy(first_line));

				if !Self::check_boundary(first_line, boundary.as_bytes()) {
					println!("multipart first line malformed");
					Err(MalformedMessage(Other))
				} else {
					println!("multipart first line ok");
					Ok(ChangePart(MultipartBody))
				}
			}
			MultipartBody => {
				let content_length = self.parse_settings.content_length
					.expect("content_length should be already set");

				let content_start_idx = self.content_start_idx
					.expect("content_start_idx should be already set");

				println!("multipart body");

				if self.inner_buffer.len() - content_start_idx > content_length {
					println!("multipart body malformed: body length");
					return Err(MalformedMessage(Other));
				}

				self.partially_parsed_request.attachments.push(HTTPAttachment::default());

				println!("multipart body ok");
				Ok(ChangePart(AttachmentHeaders))
			}
			AttachmentHeaders => {
				let header_line = match self.inner_buffer.read_line() {
					None => return Ok(Continue),
					Some(line) => line
				};

				println!("attachment headers");

				if header_line.is_empty() {
					println!("attachment headers empty line, change to attachment body");
					return Ok(ChangePart(AttachmentBody));
				}

				let mut last_attachment =
					self.partially_parsed_request.attachments.last_mut()
						.expect("at least one attachment should be already added");


				let header = parser::header_from_line(header_line)?;
				last_attachment.headers.push(header.clone());

				println!("attachment headers got header: {:?}", &header);

				match header.name.to_lowercase().as_ref() {
					"content-type" => {
						let (mime, _) =
							HTTPHeader::parse_content_type_value(header.value.as_str());
						println!("attachment headers got content-type: {mime:?}");
						last_attachment.mime_type = mime;
					}
					"content-disposition" => {
						match parser::header_content_disposition_value(header.value.as_str()) {
							None => {
								// malformed content-disposition header - skip
								println!("attachment headers malformed content-disposition");
								return Ok(Continue);
							}
							Some(v) => {
								println!("attachment headers got content-disposition: {v:?}");
								last_attachment.name = v.name;
								last_attachment.filename = v.filename;
							}
						}
					}
					_ => {}
				}

				Ok(Continue)
			}
			AttachmentBody => {
				println!("attachment body branch");

				let mut last_attachment =
					self.partially_parsed_request.attachments.last_mut()
						.expect("at least one attachment should be already added");

				let boundary = self.parse_settings
					.multipart_boundary
					.as_ref()
					.expect("multipart_boundary should be already set");

				let boundary_leading_dashes = {
					let mut tmp = b"--".to_vec();
					tmp.extend_from_slice(boundary.as_bytes());
					tmp
				};

				match self.inner_buffer.read_until(boundary_leading_dashes.as_slice()) {
					None => Ok(Continue),
					Some(data) => {
						if !check_ends_with_new_line(data) {
							println!("attachment body missing a newline at end");
							return Err(MalformedMessage(Other));
						}

						let data = strip_last_endl(data);

						last_attachment.data = data.to_vec();

						println!("attachment body ok, got data, length={}", data.len());
						Ok(ChangePart(MultipartCheckEnd))
					}
				}
			}
			MultipartCheckEnd => {
				let line = match self.inner_buffer.read_line() {
					None => return Ok(Continue),
					Some(l) => l
				};

				println!("multipart check end");

				if line.is_empty() {
					Ok(ChangePart(MultipartBody))
				} else if line.eq("--".as_bytes()) {
					println!("multipart check end, ok - finish");
					Ok(Finish)
				} else {
					println!("multipart check end, malformed");
					Err(MalformedMessage(Other))
				}
			}
		}
	}

	fn check_boundary(line: &[u8], boundary: &[u8]) -> bool {
		line.starts_with(b"--")
			&& line.get(2..).unwrap_or_default()
			.eq(boundary)
	}

	pub fn is_complete(&self) -> bool {
		self.parse_ended
	}

	pub fn debug_get_partially_parsed_request(&self) -> HTTPRequest {
		self.partially_parsed_request.clone()
	}

	pub fn get_complete_request(mut self) -> Result<HTTPRequest, HTTPParseError> {
		if !self.parse_ended {
			return Err(IncompleteRequest);
		}
		if self.parse_error.is_some() {
			return Err(self.parse_error.unwrap());
		}
		Ok(self.partially_parsed_request)
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
