use std::fs::File;
use std::io::{stdout, BufRead, Read, Write};
use std::net::{Shutdown, SocketAddr};
use std::path::Path;
use http::{HTTPHeader, HTTPRequest, HTTPResponse};

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
		return;
		match self.file_handle
			.write(payload) {
			Ok(_) => {}
			Err(e) => println!("failed to write to session log file: {e}"),
		}
	}
}

fn main() {
	examples::run_examples();

	print!("starting testing server on localhost:8500...");
	let mut listener = std::net::TcpListener::bind("localhost:8500").unwrap();
	println!("done");

	print!("initializing session debug log file...");
	let mut ses_logfile = SesLog::new(Path::new("tmp-session.log"))
		.unwrap_or_else(|e| {
			println!("failed");
			panic!("{e}");
		});
	println!("done");

	loop {
		print!("waiting for new connection...");
		let _ = stdout().flush();
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
	let mut buffer = [0; 0x4000];
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
	// check_attachments(&req);
	println!("== done check attachments ==");


	println!("== creating response ==");
	let response = create_response(&req);
	println!("== done creating response ==");

	ses_logfile.write(b"response\n<<<<<<<");
	ses_logfile.write(response.to_bytes().as_slice());
	ses_logfile.write(b">>>>>>>\n\n");

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
