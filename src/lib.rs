use std::fmt::{Debug, Display};
use std::io::Write;

pub struct URL {
	scheme: String,
	host: String,
	path: String,
	query: String,
}

pub struct HTTPHeader {
	pub name: String,
	pub value: String,
}
impl HTTPHeader {
	pub fn new(name: String, value: String) -> HTTPHeader {
		HTTPHeader { name, value }
	}
}

pub struct HTTPRequest {
	pub method: String,
	pub path: String,
	pub query: String,
	pub http_version: String,
	pub headers: Vec<HTTPHeader>,
	pub body: Vec<u8>,
}
impl Default for HTTPRequest {
	fn default() -> Self {
		Self {
			method: String::from("GET"),
			path: String::from("/"),
			query: String::new(),
			http_version: String::from("HTTP/1.1"),
			headers: vec![],
			body: Vec::<u8>::new(),
		}
	}
}
impl HTTPRequest {
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut uri = self.path.clone();
		if !self.query.is_empty() {
			uri.push('?');
			uri.push_str(&self.query);
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
			   uri,
			   self.http_version,
			   headers_joined
		)
			.expect("failed to write to ret vector");
		ret.write(&mut self.body.as_slice())
			.expect("failed to write to ret vector");

		ret
	}
}
impl Debug for HTTPRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "HTTP request (version={})", self.http_version)?;
		writeln!(f, "method={}", self.method)?;
		writeln!(f, "path={}", self.path)?;
		writeln!(f, "query=\"{}\"", self.query)?;
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
	pub fn from_str(text: &str) -> Self {
		let mut s = Self::default();
		s.push_bytes(text.as_bytes());
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
							if header_name == "content-length" {
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
		writeln!(f, "{}", self.parsed_request)?;
		Ok(())
	}
}
impl Display for HTTPPartialRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "{self:?}")
	}
}


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
