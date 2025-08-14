use crate::HTTPHeader;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct HTTPMessage {
	pub http_version: (u8, u8),
	pub headers: Vec<HTTPHeader>,
	pub body: Vec<u8>,
}

impl Default for HTTPMessage {
	fn default() -> Self {
		Self {
			http_version: (1, 1),
			headers: Vec::default(),
			body: Vec::default(),
		}
	}
}

impl HTTPMessage {
	pub fn write_headers(&self, sink: &mut dyn Write) {
		todo!()
	}

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
