use std::fmt::{Debug, Display};
use std::io::Write;
use crate::{HTTPHeader, Url, HTTPRequestAttachment, MimeType, HTTPAttachment};

pub struct HTTPRequest {
	pub method: String,
	pub url: Url,
	pub http_version: String,
	pub headers: Vec<HTTPHeader>,
	pub body: Vec<u8>,
	pub content_type: MimeType
}
impl Default for HTTPRequest {
	fn default() -> Self {
		Self {
			method: String::from("GET"),
			url: Url::default(),
			http_version: String::from("HTTP/1.1"),
			headers: vec![],
			body: Vec::<u8>::new(),
			content_type: MimeType::TextPlain
		}
	}
}
impl HTTPRequest {
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut url = self.url.path.clone();
		if !self.url.query.is_empty() {
			url.push('?');
			url.push_str(&self.url.query);
		}

		let mut found_content_length_header = false;
		let mut headers_joined = String::with_capacity(self.headers.capacity() + self.headers.len() * 4);
		for h in &self.headers {
			if h.name.to_lowercase() == "content-length" {
				found_content_length_header = true;
			}
			headers_joined.push_str(&h.name);
			headers_joined.push_str(": ");
			headers_joined.push_str(&h.value);
			headers_joined.push_str("\r\n");
		}
		if !self.body.is_empty() && !found_content_length_header {
			headers_joined.push_str(format!("content-length: {}\r\n", self.body.len()).as_str());
		}

		let mut ret = Vec::<u8>::with_capacity(0x400);
		write!(&mut ret,
			   "{} {} {}\r\n{}\r\n",
			   self.method,
			   url,
			   self.http_version,
			   headers_joined
		)
			.expect("failed to write to ret vector");
		ret.write(&mut self.body.as_slice())
			.expect("failed to write to ret vector");

		ret
	}

	pub fn get_attachments(&self) -> Option<Vec<HTTPAttachment>> {
		todo!()
	}
}
impl Debug for HTTPRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "HTTP request (version={})", self.http_version)?;
		writeln!(f, "method={}", self.method)?;
		writeln!(f, "path={}", self.url.path)?;
		writeln!(f, "query=\"{}\"", self.url.query)?;
		writeln!(f, "== headers ==")?;
		for h in &self.headers {
			writeln!(f, "-> [{}]: [{}]", h.name, h.value)?;
		}
		writeln!(f, "== body (length={}) ==", self.body.len())?;
		writeln!(f, "{:?}", String::from_utf8_lossy(self.body.as_slice()))?;
		Ok(())
	}
}
impl Display for HTTPRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", String::from_utf8_lossy(self.to_bytes().as_slice()))
	}
}
