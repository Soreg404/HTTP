use std::io::{BufRead, Read, Write};
use std::net::Shutdown;

mod tests;

use http::*;

fn example_compose_request() {
	let mut req = HTTPRequest::default();
	req.method = String::from("POST");
	req.path = String::from("/api/get-list");
	req.query = String::from("hello=world");
	req.headers.push(HTTPHeader {
		name: String::from("host"),
		value: String::from("localhost"),
	});
	req.headers.push(HTTPHeader {
		name: String::from("content-type"),
		value: String::from("text/txt"),
	});
	req.body = "Lorem ipsum\ndolor sit amet".as_bytes().to_vec();

	println!("\x1b[94m## composed request\x1b[0m");
	println!("\x1b[92m### debug\x1b[0m");
	println!("{req:?}");
	println!("\x1b[92m### display\x1b[0m");
	println!("{req}");
}

fn example_compose_response() {
	let mut res = HTTPResponse::default();
	res.status = 200;
	res.headers.push(HTTPHeader {
		name: String::from("content-type"),
		value: String::from("text/json"),
	});
	res.body = r#"{"result": "works", "number": 42}"#.as_bytes().to_vec();

	println!("\x1b[94m## composed response\x1b[0m");
	println!("\x1b[92m### debug\x1b[0m");
	println!("{res:?}");
	println!("\x1b[92m### display\x1b[0m");
	println!("{res}");
}

fn example_quick_response() {
	println!("\x1b[94m## quick response\x1b[0m");
	let resp = HTTPResponse {
		status: 418,
		..HTTPResponse::default()
	};
	println!("{resp}");
}

fn example_read_request() {
	let mut p_req = HTTPPartialRequest::default();

	p_req.push_bytes(b"POST /fi");
	p_req.push_bytes(b"nd HTTP/1.1\r");
	p_req.push_bytes(b"\nhost: local");
	p_req.push_bytes(b"host\r\n");
	p_req.push_bytes(b"content-length: 5\r\n\r\n");
	p_req.push_bytes(b"Hello");
	p_req.push_bytes(b"this won't be saved to request body (bsc content-length)");

	println!("\x1b[94m## partial request (is_complete={})\x1b[0m", p_req.is_complete());
	println!("\x1b[92m### debug\x1b[0m");
	println!("{p_req:?}");
}

fn main() {
	println!("# examples");

	example_compose_request();

	example_compose_response();

	example_quick_response();

	example_read_request();

	println!();

	println!("starting server...");
	let mut listener = std::net::TcpListener::bind("127.0.0.1:8500").unwrap();

	let mut ses_logfile = std::fs::OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open("tmp-session.bin")
		.unwrap();

	println!("server listening on port 8500.");
	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				println!("accepted a new connection");
				handle_connection(stream, &mut ses_logfile);
			}
			Err(_) => {
				println!("connection error");
				break;
			}
		}
	}

	println!("bye");
}

fn handle_connection(
	mut stream: std::net::TcpStream,
	ses_logfile: &mut std::fs::File,
) {
	let mut req = HTTPPartialRequest::default();
	ses_logfile.write(b"new connection \n<<<<<<").unwrap_or_default();

	let mut buffer = [0; 0x400];
	loop {
		println!("reading...");
		let n = stream.read(&mut buffer).expect("failed to read");
		println!("read {} bytes", n);

		if n == 0 {
			println!("connection closed by peer");
			break;
		}

		ses_logfile.write(&buffer[0..n]).unwrap_or_default();

		req.push_bytes(buffer[..n].as_ref());

		if req.is_complete() {
			break;
		}
	}

	println!("{:?}", req);
	ses_logfile.flush().unwrap();
	ses_logfile.write(b">>>>>>\n\n").unwrap();

	if !req.is_complete() {
		println!("incomplete request, bail");
		return;
	}

	println!("responding...");

	let req = req.get_complete_request().unwrap();
	let response = match req.path.as_str() {
		"/file-form" => {
			let html = std::fs::read_to_string("html/file-form.html").unwrap();
			HTTPResponse {
				body: html.as_bytes().to_vec(),
				..HTTPResponse::default()
			}
		}
		"/file-form-result" => {
			HTTPResponse::new_short(200)
		}
		_ => HTTPResponse::new_short(404)
	};

	if response.body.len() < 60 {
		println!("response: {:?}", &response);
	}

	let result = stream.write(response.to_bytes().as_slice());
	if result.is_err() {
		println!("write error");
	}

	stream.flush().expect("failed to flush");

	println!("closing connection...");
	stream.shutdown(Shutdown::Write).expect("failed to close connection");
	stream.shutdown(Shutdown::Read).expect("failed to close connection");
	println!("done");
}
