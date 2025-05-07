use std::io::{BufRead, Read, Write};
use std::net::Shutdown;

mod examples;

fn main() {
	examples::run_examples();

	println!("starting server...");
	let mut listener = std::net::TcpListener::bind("127.0.0.1:8500").unwrap();

	let mut ses_logfile = std::fs::OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open("../tmp-session.tmp")
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
	let mut req = http::HTTPPartialRequest::default();
	ses_logfile.write(b"new connection \n<<<<<<<").unwrap_or_default();

	let mut buffer = [0; 0x400];
	loop {
		// println!("reading...");
		let n = stream.read(&mut buffer).expect("failed to read");
		// println!("read {} bytes", n);

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

	// println!("{:?}", req);
	ses_logfile.flush().unwrap();
	ses_logfile.write(b">>>>>>>\n\n").unwrap();

	if !req.is_complete() {
		println!("incomplete request, bail");
		return;
	}

	println!("responding...");

	let req = req.get_complete_request().unwrap();

	println!("attachments test: {:?}", attachments_test(&req));

	let response = match req.url.path.as_str() {
		"/file-form" => {
			let html = std::fs::read_to_string("html/file-form.html").unwrap();
			http::HTTPResponse {
				body: html.as_bytes().to_vec(),
				..http::HTTPResponse::default()
			}
		}
		"/file-form-result" => {
			http::HTTPResponse::quick(200)
		}
		_ => http::HTTPResponse::quick(404)
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

fn attachments_test(req: &http::HTTPRequest) -> Result<(), &str> {
	let content_type_header = req.headers
		.iter().find(|h| h.name.to_lowercase().eq("content-type"));

	if content_type_header.is_none() { return Err("missing content-type header"); }
	let content_type_header = content_type_header.unwrap();

	// Content-Type: multipart/form-data; boundary=----WebKitFormBoundaryaCMjE5pYm5kWl5MB

	let pos = content_type_header.value.find(';');
	if pos.is_none() { return Err("missing ';'"); }
	let pos = pos.unwrap();
	let (mime_type, boundary) = content_type_header.value.split_at(pos);
	let mime_type = mime_type.trim();
	let boundary = boundary[1..].trim();

	if !mime_type.to_lowercase().eq("multipart/form-data") {
		return Err("mime type is not multipart/form-data");
	}
	if !boundary.starts_with("boundary=") { return Err("invalid value"); }

	let boundary = boundary.split_at("boundary=".len()).1.as_bytes();

	println!("multipart with boundary {:?}", boundary);

	for (i, window) in req.body.windows(boundary.len()).enumerate() {
		if window != boundary {
			continue;
		}

		println!("found boundary on i={i}");
	}


	// for part in b {
	// 	println!("part len: {}", part.len());
	// 	if part.len() < 400 {
	// 		println!("part: {:?}", part);
	// 	}
	// }

	Ok(())
}
