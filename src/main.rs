use std::fmt::format;
use std::fs::File;
use std::io::{stdout, BufRead, Read, Write};
use std::net::{Shutdown, SocketAddr};
use std::path::Path;
use http::{HTTPHeader, HTTPPartialRequest, HTTPRequest, HTTPResponse};

mod examples;

struct Log {
	file_handle: File,
}

impl Log {
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
			Ok(_) => {}
			Err(e) => println!("failed to write to log file: {e}"),
		}
	}
}

fn main() {
	examples::run_examples();

	let server_thread_handle = std::thread::spawn(start_sample_server);

	let mut con = std::net::TcpStream::connect("example.com:80").unwrap();
	println!("{con:?}");
	let mut req = HTTPRequest::default();
	req.method = String::from("GET");
	// req.url = ;
	req.headers = vec![
		HTTPHeader::new("host".into(), "example.com".into()),
	];
	let req_bytes = req.to_bytes();
	println!("{:?}", &req_bytes);
	con.write_all(&req_bytes).unwrap();

	// let mut response_collector = HTTPPartialRe::default();
	// let mut buffer = [0u8; 0x400];
	// while !response_collector.


	println!("main finished, waiting for server thread to join in.");
	server_thread_handle.join().unwrap();
}

fn start_sample_server() {
	print!("starting testing server on localhost:8500...");
	let mut listener = std::net::TcpListener::bind("localhost:8500").unwrap();
	println!("done");

	print!("initializing server session log file...");
	let mut log = Log::new(Path::new("server-session.log"))
		.unwrap_or_else(|e| {
			println!("failed");
			panic!("{e}");
		});
	println!("done");

	println!("sample server loop starts.");
	loop {
		match listener.accept() {
			Ok((mut stream, peer))
			=> handle_connection(&mut stream, &peer, &mut log),

			Err(e)
			=> println!("server failed to accept connection: {:?}", e)
		}
	}
}

fn handle_connection(
	stream: &mut std::net::TcpStream,
	peer: &SocketAddr,
	log: &mut Log,
) {
	let mut req = http::HTTPPartialRequest::default();
	log.write(format!("handling connection from {peer}\n").as_bytes());

	log.write("collecting request...\n<<<<<<<".as_bytes());
	let mut buffer = [0; 0x400];
	while !req.is_complete() {
		let n = stream.read(&mut buffer)
			.expect("failed to read stream");

		if n == 0 {
			log.write(">>>>>>>\nfail: connection lost before request completion".as_bytes());
			return;
		}
		log.write(&buffer[..n]);

		req.push_bytes(&buffer[..n]);
	}
	log.write(">>>>>>>\ndone collecting request\n".as_bytes());

	let req = req.get_complete_request()
		.expect("should be completed here");
	log.write(format!("complete request debug view: {:?}\n", &req).as_bytes());

	log.write("check attachments (wip)...\n".as_bytes());
	// check_attachments(&req);
	log.write("done check attachments\n".as_bytes());


	log.write("creating response...\n".as_bytes());
	let response = create_response(&req);
	log.write("done creating response\n".as_bytes());

	log.write("response bytes\n<<<<<<<".as_bytes());
	log.write(response.to_bytes().as_slice());
	log.write(">>>>>>>\n".as_bytes());

	match stream.write(response.to_bytes().as_slice()) {
		Ok(count) => {
			log.write(format!("wrote {count} bytes to peer\n").as_bytes());
		}
		Err(e) => {
			log.write(format!("failed to respond to peer: {e}\n").as_bytes());
		}
	};

	log.write("closing connection...\n".as_bytes());
	stream.flush()
		.unwrap_or_else(|e| log.write(format!("failed to flush: {e}\n").as_bytes()));
	stream.shutdown(Shutdown::Write)
		.unwrap_or_else(|e| log.write(format!("failed to shutdown write: {e}\n").as_bytes()));
	stream.shutdown(Shutdown::Read)
		.unwrap_or_else(|e| log.write(format!("failed to shutdown read: {e}\n").as_bytes()));
	log.write("connection closed\n\n".as_bytes());
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
			let attachment =
				match req.attachments
					.iter()
					.find(|a| a.name == "file") {
					Some(a) => a,
					None => return HTTPResponse::quick(400)
				};

			let mut msg =
				format!(
					"<h1>submitted a file!</h1>\n\
					<div><a href='/file-form'>submit another!</a></div>
					<span><u>name:</u>&nbsp;{:?}</span><br>\n\
					<span><u>size:</u>&nbsp;{}</span>\n",
					attachment.filename,
					attachment.data.len()
				);

			let path = Path::new("upload.jpg");

			match std::fs::write(&path, &attachment.data) {
				Ok(_) => {
					msg.push_str(
						format!("<div>successfully saved to <code>{path:?}</code>!</div>\n")
							.as_str()
					)
				}
				Err(e) => {
					msg.push_str(
						format!("<div>failed to save file!</div>\n<div>{e}</div>\n").as_str()
					)
				}
			};

			msg.push_str(r#"<div>lookie lookie!</div>"#);
			msg.push('\n');

			msg.push_str(r#"<div><img src="/upload" width="500" alt="the image from /upload"></div>"#);
			msg.push('\n');

			HTTPResponse {
				body: msg.as_bytes().to_vec(),
				..Default::default()
			}
		}
		"/upload" => {
			match std::fs::read("upload.jpg") {
				Ok(file_data) => {
					HTTPResponse {
						headers: vec![
							HTTPHeader::new(
								"content-type".to_string(),
								"image/jpeg".to_string()
							)
						],
						body: file_data,
						..Default::default()
					}
				}
				Err(e) => HTTPResponse::quick(404)
			}
		}
		_ => HTTPResponse::quick(404)
	};

	if response.body.len() < 60 {
		println!("response debug view: {:?}", &response);
	}

	response
}

/*fn check_attachments(req: &HTTPRequest) {

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
*/
