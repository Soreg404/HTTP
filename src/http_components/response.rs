use std::fmt::{Debug, Display};
use std::io::Write;
use crate::HTTPHeader;

pub struct HTTPResponse {
	pub http_version: String,
	pub status: usize,
	pub headers: Vec<HTTPHeader>,
	pub body: Vec<u8>,
}
impl Default for HTTPResponse {
	fn default() -> Self {
		Self {
			http_version: String::from("HTTP/1.1"),
			status: 200,
			headers: vec![],
			body: Vec::<u8>::new(),
		}
	}
}
impl HTTPResponse {
	pub fn new_short(status_code: usize) -> Self {
		Self {
			status: status_code,
			..Default::default()
		}
	}
	fn status_code_str(status_code: usize) -> &'static str {
		match status_code {
			200 => "OK",
			404 => "NOT FOUND",
			500 => "INTERNAL SERVER ERROR",
			418 => "I'M A TEAPOT",
			_ => ""
		}
	}
	pub fn to_bytes(&self) -> Vec<u8> {
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
			   self.http_version,
			   self.status.to_string(),
			   Self::status_code_str(self.status),
			   headers_joined
		)
			.expect("failed to write to ret vector");
		ret.write(&mut self.body.as_slice())
			.expect("failed to write to ret vector");

		ret
	}
}
impl Debug for HTTPResponse {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "HTTP response (version={})", self.http_version)?;
		writeln!(f, "status={} (\"{}\")", self.status, Self::status_code_str(self.status))?;
		writeln!(f, "== headers ==")?;
		for h in &self.headers {
			writeln!(f, "-> [{}]: [{}]", h.name, h.value)?;
		}
		writeln!(f, "== body (length={}) ==", self.body.len())?;
		writeln!(f, "{}", String::from_utf8_lossy(&self.body).to_string())?;
		Ok(())
	}
}
impl Display for HTTPResponse {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "{}", String::from_utf8_lossy(self.to_bytes().as_slice()))
	}
}
