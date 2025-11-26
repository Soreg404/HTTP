use std::cmp::{max, min};
use crate::proto::internal::buffer_reader::BufferReaderRef;
use crate::proto::internal::message::HTTPMessage;
use crate::proto::internal::parser_header::ContentType;
use crate::HTTPParseError::{MalformedMessage, MissingContentTypeHeader};
use crate::{HTTPAsciiStr, HTTPAttachment, HTTPHeader, HTTPParseError, MalformedMessageKind};
use std::str::FromStr;
use HTTPParseError::MissingMultipartBoundary;
use MalformedMessageKind::MalformedMultipartBody;

#[derive(Debug, Clone)]
pub struct HTTPMessageMultipart {
	pub http_version: (u8, u8),
	pub headers: Vec<HTTPHeader>,
	pub attachments: Vec<HTTPAttachment>,
}

#[cfg(feature = "bench")]
impl Default for HTTPMessageMultipart {
	fn default() -> Self {
		Self {
			http_version: (1, 1),
			headers: Vec::default(),
			body: Vec::default(),
		}
	}
}

impl TryFrom<HTTPMessage> for HTTPMessageMultipart {
	type Error = HTTPParseError;
	fn try_from(message: HTTPMessage) -> Result<Self, Self::Error> {
		println!("multipart message try from message");

		let header = match message.headers.iter().find(|h| {
			h.name.eq_ignore_ascii_case("content-type")
		}) {
			Some(v) => v,
			None => return Err(MissingContentTypeHeader)
		};

		println!("content type header: {:?}", header);

		let content_type = ContentType::try_from(header.value.as_str())?;
		println!("content type: {:?}", content_type);

		let boundary = match content_type.boundary {
			Some(v) => v,
			None => return Err(MissingMultipartBoundary)
		};

		let attachments = parse_multipart_body(
			message.body.as_slice(), boundary.as_bytes())?;

		Ok(
			HTTPMessageMultipart {
				http_version: message.http_version,
				headers: message.headers,
				attachments,
			}
		)
	}
}

fn parse_multipart_body(body: &[u8], boundary: &[u8])
						-> Result<Vec<HTTPAttachment>, HTTPParseError> {
	match basic_tmp_parse_multipart_body(body, boundary) {
		Some(v) => Ok(v),
		None => Err(MalformedMessage(MalformedMultipartBody))
	}
}

fn basic_tmp_parse_multipart_body(body: &[u8], boundary: &[u8])
								  -> Option<Vec<HTTPAttachment>> {
	let slice = body
		.strip_prefix(b"--")?
		.strip_prefix(boundary)?;

	let slice = slice.strip_prefix(b"\r").unwrap_or(slice)
		.strip_prefix(b"\n")?;

	let slice = slice.strip_suffix(b"\n")?;
	let slice = slice.strip_suffix(b"\r").unwrap_or(slice)
		.strip_suffix(b"--")?;

	let mut ret = Vec::new();

	let mut start_index = 0;
	for i in 0..slice.len() {
		let subpart = &slice[start_index..=i];

		let subpart = match subpart.strip_suffix(boundary) {
			Some(v) => {
				start_index = i + 1;
				v
			},
			None => continue
		};

		let subpart = subpart.strip_suffix(b"\n--")?;
		let subpart = subpart.strip_suffix(b"\r").unwrap_or(subpart);

		let mut subpart_buffer_reader = BufferReaderRef::new(subpart);

		let mut current_attachment = HTTPAttachment {
			headers: Vec::new(),
			body: Vec::new(),
		};

		loop {
			let line = subpart_buffer_reader.take_line()?;

			let line = match HTTPAsciiStr::try_from(line) {
				Ok(line) => line,
				Err(e) => return None //Some(Err(e))
			};

			let line = line.trim();

			if line.is_empty() {
				break;
			}

			let header = match HTTPHeader::from_str(line) {
				Ok(header) => header,
				Err(e) => return None //Some(Err(e))
			};

			current_attachment.headers.push(header);
		}

		current_attachment.body.extend_from_slice(subpart_buffer_reader.take_all()?);

		ret.push(current_attachment);
	}

	Some(ret)
}
