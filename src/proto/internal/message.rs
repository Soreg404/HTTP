use std::io::Write;
use crate::proto::attachment::HTTPAttachment;
use crate::proto::header::HTTPHeader;
use crate::proto::mime_type::MimeType;

#[derive(Clone, Debug)]
pub struct HTTPMessage {
	pub http_version: String,
	pub headers: Vec<HTTPHeader>,
	pub mime_type: MimeType,
	pub body: Vec<u8>,
	pub attachments: Vec<HTTPAttachment>,
}

impl HTTPMessage {
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut headers_joined = String::with_capacity(
			self.headers.capacity() + self.headers.len() * 4);

		if !self.body.is_empty() {
			headers_joined.push_str(
				format!("content-length: {}\r\n", self.body.len()).as_str());
		}

		for h in &self.headers {
			if h.name.to_lowercase() == "content-length" {
				continue;
			}
			headers_joined.push_str(&h.name);
			headers_joined.push_str(": ");
			headers_joined.push_str(&h.value);
			headers_joined.push_str("\r\n");
		}

		headers_joined.push_str("\r\n");

		let mut ret = Vec::<u8>::with_capacity(
			headers_joined.len() + self.body.len());

		ret.write(headers_joined.as_bytes())
			.expect("writing bytes to vec");

		ret.write(self.body.as_slice())
			.expect("writing bytes to vec");

		ret
	}
}


impl Default for HTTPMessage {
	fn default() -> Self {
		Self {
			http_version: String::from("HTTP/1.1"),
			headers: Vec::default(),
			mime_type: MimeType::TextPlain,
			body: Vec::default(),
			attachments: Vec::default(),
		}
	}
}
