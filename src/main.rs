use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::net::{Shutdown, SocketAddr};
use std::path::Path;
use http::{HTTPRequest, HTTPResponse};

mod examples;

struct SesLog {
	file_handle: File,
}

impl SesLog {
	fn new(path: &Path) -> std::io::Result<Self> {
		Ok(
			Self {
				file_handle: std::fs::OpenOptions::new()
					.create(true)
					.write(true)
					.truncate(true)
					.open(path)?
			}
		)
	}
	fn write(&mut self, payload: &[u8]) {
		match self.file_handle
			.write(payload) {
			Err(e) => println!("failed to write to session log file: {e}"),
			_ => {}
		}
	}
}

fn main() {
	examples::run_examples();

	print!("starting testing server on localhost:8500...");
	let mut listener = std::net::TcpListener::bind("localhost:8500").unwrap();
	println!("done");

	print!("initializing session debug log file...");
	let mut ses_logfile = SesLog::new(Path::new("../tmp-session.tmp"))
		.unwrap_or_else(|e| {
			println!("failed");
			panic!(e);
		});
	println!("done");

	loop {
		print!("waiting for new connection...");
		let mut client_socket = listener.accept();
		match client_socket {
			Ok((mut stream, peer)) => {
				println!("accepted {:?}", &peer);

				println!("=== handling connection ===");
				handle_connection(&mut stream, &peer, &mut ses_logfile);
				println!("=== done handling connection ===");
				println!();
			}
			Err(e) => {
				println!("failed with error {:?}", e);
			}
		}
	}
}

fn handle_connection(
	stream: &mut std::net::TcpStream,
	peer: &SocketAddr,
	ses_logfile: &mut SesLog,
) {
	let mut req = http::HTTPPartialRequest::default();
	ses_logfile.write(format!("new connection from {peer}\n<<<<<<<").as_bytes());

	print!("collecting request...");
	let mut buffer = [0; 0x400];
	while !req.is_complete() {
		let n = stream.read(&mut buffer)
			.expect("failed to read stream");

		if n == 0 {
			println!("failed; connection lost before request completion");
			return;
		}
		ses_logfile.write(&buffer[0..n]);

		req.push_bytes(&buffer[..n]);
	}
	println!("done");

	let req = req.get_complete_request()
		.expect("should be completed here");
	println!("complete request debug view: {:?}", &req);

	ses_logfile.write(b">>>>>>>\n\n");

	println!("== check attachments ==");
	check_attachments(&req);
	println!("== done check attachments ==");


	println!("== creating response ==");
	let response = create_response(&req);
	println!("== done creating response ==");

	print!("responding...");
	match stream.write(response.to_bytes().as_slice()) {
		Ok(count) => {
			println!("done: wrote {count} bytes");
		}
		Err(e) => {
			println!("failed: {e}");
		}
	};

	print!("closing connection...");
	stream.flush()
		.unwrap_or_else(|e| println!("failed to flush: {e}"));
	stream.shutdown(Shutdown::Write)
		.unwrap_or_else(|e| println!("failed to shutdown write: {e}"));
	stream.shutdown(Shutdown::Read)
		.unwrap_or_else(|e| println!("failed to shutdown read: {e}"));
	println!("done");
}

fn create_response(req: &HTTPRequest) -> HTTPResponse {
	let response = match req.url.path_raw.as_str() {
		"/file-form" => {
			match std::fs::read_to_string("html/file-form.html") {
				Ok(html) => {
					HTTPResponse {
						body: html.as_bytes().to_vec(),
						status: 200,
						..HTTPResponse::default()
					}
				}
				Err(e) => {
					HTTPResponse {
						body: b"failed to load html/file-form.html".to_vec(),
						status: 500,
						..HTTPResponse::default()
					}
				}
			}
		}
		"/file-form-result" => {
			HTTPResponse::quick(200)
		}
		_ => HTTPResponse::quick(404)
	};

	if response.body.len() < 60 {
		println!("response debug view: {:?}", &response);
	}

	response
}

fn check_attachments(req: &HTTPRequest) {

	req.headers
		.all_headers_raw
		.iter()
		.find(|h| h.name.to_lowercase().eq("content-type"));

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
