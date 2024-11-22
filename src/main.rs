use std::io::{BufRead, Read, Write};
use std::net::Shutdown;

mod my_http;
use my_http::*;

fn example_compose_request() {
	let mut req = HTTPRequest::default();
	req.method = String::from("POST");
	req.path = String::from("/api/get-list");
	req.query = String::from("hello=world");
	req.headers.push(HTTPHeader {
		name: String::from("host"),
		value: String::from("localhost")
	});
	req.headers.push(HTTPHeader {
		name: String::from("content-type"),
		value: String::from("text/txt")
	});
	req.body = "Lorem ipsum dolor sit amet".as_bytes().to_vec();

	println!("== composed request ==");
	println!("-> debug");
	println!("{req:?}");
	println!("-> to_bytes/text");
	println!("{}", req.to_bytes());
}

fn main() {
	println!("=== examples ===");
	example_compose_request();

	println!();

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
	let mut req = HTTPPartialRequest::new();

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
	// stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).expect("failed to write");
	{
		let mut resp = HTTPResponse::default();
		let img_name = req.parsed_request.query;
		if img_name.find('\\').is_some() {
			let r = HTTPResponse {
				http_version: "HTTP/1.1".to_string(),
				status: 400,
				headers: vec![
					HTTPHeader::new("content-type".to_string(), "text/txt".to_string())
				],
				body: "incorrect input".as_bytes().to_vec(),
			};
			stream.write(&r.to_bytes()).unwrap();
		} else {
			let path = format!("C:\\!\\Temp\\hm\\{}", img_name);
			resp.body = std::fs::read(path).unwrap_or_else(|e| e.to_string().into_bytes());
			stream.write(&resp.to_bytes()).expect("failed to write");
		}
	}
	stream.flush().expect("failed to flush");

	println!("closing connection...");
	stream.shutdown(Shutdown::Write).expect("failed to close connection");
	stream.shutdown(Shutdown::Read).expect("failed to close connection");
	println!("done");
}
