use http::HTTPHeader;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

// mod examples;

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

	let mut con = TcpStream::connect("http.badssl.com:80").unwrap();

	let mut req = http::HTTPRequest::default();
	req.set_method("GET");
	req.set_target("/");
	req.headers_mut().push(HTTPHeader::new("host".into(), "http.badssl.com".into()));

	con.write_all(req.to_bytes().as_slice()).unwrap();

	let mut p_res = http::HTTPPartialResponse::default();
	while !p_res.is_finished() {
		let mut buf = [0; 512];
		let len = con.read(&mut buf).unwrap();
		if len == 0 {
			p_res.signal_connection_closed();
			break;
		}
		p_res.push_bytes(&buf[..len]);
	}

	match p_res.into_response() {
		Ok(res) => {
			println!("{res}");
		}
		Err(e) => {
			println!("failed: {e:?}");
		}
	};




	// examples::run_examples();

	// let server_thread_handle = std::thread::spawn(start_sample_server);

	// test_collect_response();

	// println!("main finished, waiting for server thread to join in.");
	// server_thread_handle.join().unwrap();

	// start_sample_server();
}

#[cfg(feature="bench")]
fn test_collect_response() {

	let mut req = HTTPRequest::default();

	req.get_mime_type();

	req.set_mime_type(MimeType::TextPlain);



	println!("connecting");
	let mut con = std::net::TcpStream::connect("127.0.0.1:80").unwrap();
	println!("{con:?}");
	let mut req = HTTPRequest::default();
	req.method = String::from("GET");
	req.url = Url::from_str("/dashboard/images/favicon.png").unwrap();
	req.message.headers = vec![
		HTTPHeader::new("host".into(), "localhost".into()),
	];
	println!("{req}");
	let req_bytes = req.to_bytes();
	con.write_all(&req_bytes).unwrap();

	println!("new message collector");
	let mut response_collector = HTTPPartialResponse::default();
	let mut buffer = [0u8; 0x400];
	while !response_collector.is_complete() {
		let len = con.read(&mut buffer).unwrap();

		println!("push {len}");
		response_collector.push_bytes(&buffer[..len]);
	}

	println!("convert");
	let response = response_collector.into_response();

	println!("GOT RESPONSE:\n\n{response}");

	std::fs::write("favicon.png", response.message.body).unwrap()

}

#[cfg(feature="bench")]
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

#[cfg(feature="bench")]
fn handle_connection(
	stream: &mut std::net::TcpStream,
	peer: &SocketAddr,
	log: &mut Log,
) {
	println!("handling connection from {peer}");
	log.write(format!("handling connection from {peer}\n").as_bytes());

	let mut req = MessageParser::new_request();

	// let mut req = http::HTTPPartialRequest::default();

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

	// println!("debug partially parsed request: {}",
	// 		 req.debug_get_partially_parsed_request());

	// let req = match req.into_request() {
	// 	Ok(r) => r,
	// 	Err(e) => {
	// 		println!("partial request error: {e:?}");
	// 		return;
	// 	}
	// };
	let req = req.into_request();
	log.write(format!("complete request debug view: {:?}\n", &req).as_bytes());

	log.write("check request attachments (wip)...\n".as_bytes());
	// check_attachments(&req);
	log.write("done check request attachments\n".as_bytes());


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

#[cfg(feature="bench")]
fn create_response(req: &HTTPRequest) -> HTTPResponse {
	println!("generating response");

	let response = match req.url.path.as_str() {
		"/file-form" => {
			match std::fs::read_to_string("html/file-form.html") {
				Ok(html) => {
					HTTPResponse {
						status_code: 200,
						message: HTTPMessage {
							body: html.as_bytes().to_vec(),
							..Default::default()
						},
						..HTTPResponse::default()
					}
				}
				Err(e) => {
					HTTPResponse {
						status_code: 500,
						message: HTTPMessage {
							body: b"failed to load html/file-form.html".to_vec(),
							..Default::default()
						},
						..HTTPResponse::default()
					}
				}
			}
		}
		"/file-form-result" => {
			let attachment =
				match req.message.attachments
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
					attachment.body.len()
				);

			let path = Path::new("upload.jpg");

			match std::fs::write(&path, &attachment.body) {
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
				status_code: 200,
				message: HTTPMessage {
					body: msg.as_bytes().to_vec(),
					..Default::default()
				},
				..HTTPResponse::default()
			}
		}
		"/upload" => {
			match std::fs::read("upload.jpg") {
				Ok(file_data) => {
					HTTPResponse {
						status_code: 200,
						message: HTTPMessage {
							headers: vec![
								HTTPHeader::new(
									"content-type".to_string(),
									"image/jpeg".to_string()
								)
							],
							body: file_data,
							..Default::default()
						},
						..HTTPResponse::default()
					}
				}
				Err(e) => HTTPResponse::quick(404)
			}
		}

		_ => {
			println!("quick");
			HTTPResponse::quick(404)
		}
	};

	if response.message.body.len() < 60 {
		println!("response debug view: {:?}", &response);
	}

	response
}

#[cfg(feature="bench")]
fn check_attachments(req: &HTTPRequest) {
	println!("attachments: {:?}", &req.message.attachments);
}
