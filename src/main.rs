use std::io::{Read, Write};
use std::net::TcpListener;
use http::consts::StatusCode;

fn main() {
	let listener = TcpListener::bind("[::1]:48001").unwrap();
	'accept_connection: loop {
		let (mut tcp_stream, peer) = listener.accept()
			.unwrap();

		println!("accepted connection from {:#?}", peer);
		println!("(local address: {:#?})", tcp_stream.local_addr());


		let mut partial_request = http::request::Collector::new();

		'collect_request: loop {
			let mut buff = [0; 0xff];
			match tcp_stream.read(&mut buff) {
				Ok(n) => {
					if n == 0 {
						continue 'accept_connection;
					}

					println!("pushing bytes: {:?}", String::from_utf8_lossy(&buff[..n]));

					let n = partial_request.push_bytes(&buff[..n]);

					println!("consumed {n} bytes");
				}
				Err(e) => {
					eprintln!("tcp_stream.read() error: {e:#?}");
					continue 'collect_request;
				}
			}

			if partial_request.is_finished() {
				println!("is finished");
				break 'collect_request;
			}
		}

		let request = match partial_request.into_request() {
			Ok(v) => v,
			Err(e) => {
				eprintln!("http request parse error: {e:#?}");
				continue 'accept_connection;
			}
		};

		dbg!(request);

		let mut builder = http::response::Builder::new();
		builder.set_status(StatusCode::IM_A_TEAPOT);
		builder.push_header("helo", "world");

		let bytes = builder
			.into_response()
			.into_bytes();

		// tcp_stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap()
		tcp_stream.write_all(bytes.as_slice()).unwrap()
	}
}
