use std::io::{BufRead, Read, Write};
use std::net::Shutdown;

mod my_http;

pub struct HTTPHeader {
	name: String,
	value: String,
}

pub struct Request {
	pub part_counter: u8,
	is_complete: bool,
	internal_buffer: Vec<u8>,

	new_line_hold: bool,

	method: String,
	path: String,
	query: String,
	http_version: String,
	headers: Vec<HTTPHeader>,
	body: String,
	content_length: usize,
}

impl Request {
	pub fn new() -> Self {
		Request {
			part_counter: 0,
			is_complete: false,
			internal_buffer: Vec::with_capacity(0x400),

			new_line_hold: false,

			method: String::new(),
			path: String::new(),
			query: String::new(),
			http_version: String::new(),
			headers: Vec::new(),

			content_length: 0,
			body: String::new(),
		}
	}
	pub fn from_str(text: &str) -> Self {
		let mut s = Self::new();
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
						0 => self.method = buffer.iter().collect(),
						1 => {
							let uri: String = buffer.iter().collect();
							let mut uri_it = uri.split('?');
							self.path = uri_it.next().unwrap().to_string();
							self.query = match (uri_it.next()) {
								Some(v) => String::from(v),
								None => String::new(),
							};
						}

						2 => self.http_version = buffer.iter().collect(),
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

							self.headers.push(HTTPHeader {
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
				self.body = String::from_utf8_lossy(self.internal_buffer.as_slice()).to_string();
				self.internal_buffer.clear();
				self.is_complete = true;
			}
		}
	}
}

impl std::fmt::Display for Request {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "HTTP request (version={}) /{}",
				 self.http_version,
				 if self.is_complete { "complete" } else { "incomplete" })?;
		writeln!(f, "method={}", self.method)?;
		writeln!(f, "path={}", self.path)?;
		writeln!(f, "query=\"{}\"", self.query)?;
		writeln!(f, "== headers ==")?;
		for h in &self.headers {
			writeln!(f, "-> [{}]: [{}]", h.name, h.value)?;
		}
		writeln!(f, "== body (length={}) ==", self.content_length)?;
		writeln!(f, "{}", self.body)?;
		Ok(())
	}
}

fn main() {
	println!("starting server...");
	let mut listener = std::net::TcpListener::bind("127.0.0.1:8500").unwrap();

	println!("server listening on port 8500.");
	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				println!("accepted a new connection");
				handle_connection(stream);
			}
			Err(e) => {
				println!("connection error");
				break;
			}
		}
	}

	println!("bye");
}

fn handle_connection(mut stream: std::net::TcpStream) {
	let mut req = Request::new();

	let mut buffer = [0; 0x400];
	loop {
		println!("reading...");
		let n = stream.read(&mut buffer).expect("failed to read");
		println!("read {} bytes", n);

		if n == 0 {
			println!("connection closed by peer");
			break;
		}

		req.push_bytes(buffer[..n].as_ref());

		if req.is_complete {
			break;
		}
	}

	println!("{}", req);

	println!("responding...");
	stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).expect("failed to write");
	stream.flush().expect("failed to flush");

	println!("closing connection...");
	stream.shutdown(Shutdown::Write).expect("failed to close connection");
	stream.shutdown(Shutdown::Read).expect("failed to close connection");
	println!("done");
}
