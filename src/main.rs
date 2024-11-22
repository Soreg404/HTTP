use std::io::{BufRead, Read, Write};

mod my_http;

pub struct Request {
	pub part_counter: u8,
	request_line_byte_buffer: Vec<u8>,
	headers_byte_buffer: Vec<u8>,
	body_byte_buffer: Vec<u8>,

	is_new_line: bool,
	method: String,
	path: String,
	query: String,
	headers: Vec<String>,
	body: String,
	version: String,
}

impl Request {
	pub fn new() -> Self {
		Request {
			part_counter: 0,
			request_line_byte_buffer: Vec::new(),
			headers_byte_buffer: Vec::new(),
			body_byte_buffer: Vec::new(),
			is_new_line: false,

			method: String::new(),
			path: String::new(),
			query: String::new(),
			version: String::new(),
			headers: Vec::new(),
			body: String::new(),
		}
	}
	pub fn from_str(text: &str) -> Self {
		let mut s = Self::new();
		s.push_bytes(text.as_bytes());
		s.parse();
		s
	}

	fn parse_request_first_line(&mut self) -> Result<(), &'static str> {
		let line_str = String::from_utf8_lossy(
			self.request_line_byte_buffer.as_slice())
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

						2 => self.version = buffer.iter().collect(),
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

	fn parse_request_headers(&mut self) -> Result<(), &'static str> {
		let mut header_buffer = Vec::<u8>::with_capacity(0x400);
		let mut cr_hold = false;
		for (i, c) in self.headers_byte_buffer.iter().enumerate() {
			if *c == b'\r' {
				continue;
			}
			if *c == b'\n' {
				if !cr_hold {
					cr_hold = true;
					self.headers.push(
						String::from_utf8_lossy(header_buffer.as_slice()).to_string());
					header_buffer.clear();
				}
			} else {
				header_buffer.push(*c);
				cr_hold = false;
			}
		}
		Ok(())
	}

	pub fn push_bytes(&mut self, buffer: &[u8]) {
		for c in buffer {
			match self.part_counter {
				0 => self.request_line_byte_buffer.push(*c),
				1 => self.headers_byte_buffer.push(*c),
				2 => self.body_byte_buffer.push(*c),
				_ => panic!("invalid part_counter ({})", self.part_counter)
			}

			if *c == b'\n' {
				match self.part_counter {
					0 => {
						println!("push_bytes, \\n - parse first line");
						self.parse_request_first_line()
							.expect("bad first line");
						self.part_counter = 1;
						self.is_new_line = true;
					}
					1 => {
						if self.is_new_line {
							self.part_counter = 2;
							self.parse_request_headers().expect("bad headers");
						} else {
							self.is_new_line = true;
						}
					}
					_ => {}
				}
			} else if *c != b'\r' {
				self.is_new_line = false;
			}
		}
	}

	pub fn parse(&mut self) -> Result<(), &'static str> {
		if self.part_counter != 2 {
			return Err("incomplete request.");
		}

		println!("request");
		println!("method={}", self.method);
		println!("path={}", self.path);
		println!("query={}", self.query);
		println!("version={}", self.version);

		println!("== headers ({}) ==", self.headers.len());
		for h in self.headers.iter() {
			println!("-> {}", h);
		}

		println!("== body ==");
		println!("{}", self.body);

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
			println!("connection closed");
			break;
		}

		req.push_bytes(buffer[..n].as_ref());

		if req.part_counter >= 2 {
			break;
		}
	}

	req.parse().expect("failed to parse");

	stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).expect("failed to write");
	println!("done");
}
