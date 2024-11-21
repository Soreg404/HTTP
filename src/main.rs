use std::io::{Read, Write};

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
					0 => self.part_counter = 1,
					1 => {
						if self.is_new_line {
							self.part_counter = 2
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
		println!("req:\n1st line: {}\n== headers ==\n{}\n== body ==\n{}",
				 String::from_utf8_lossy(self.request_line_byte_buffer.as_slice()),
				 String::from_utf8_lossy(self.headers_byte_buffer.as_slice()),
				 String::from_utf8_lossy(self.body_byte_buffer.as_slice())
		);
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
		let n = stream.read(&mut buffer).expect("failed to read");

		req.push_bytes(buffer[..n].as_ref());

		if req.part_counter >= 2 {
			break;
		}
	}

	req.parse().expect("failed to parse");

	stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).expect("failed to write");
	println!("done");
}
