use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
	let listener = TcpListener::bind("[::1]:48002").unwrap();
	'accept_connection: loop {
		let (mut tcp_stream, peer) = listener.accept()
			.unwrap();

		println!("accepted connection from {:#?}", peer);
		println!("(local address: {:#?})", tcp_stream.local_addr());

		let mut partial_request = http::RequestCollector::new();

		'collect_request: loop {
			let mut buff = [0; 0xff];
			match tcp_stream.read(&mut buff) {
				Ok(n) => {
					if n == 0 {
						continue 'accept_connection;
					}

					partial_request.push_bytes(&buff[..n]);
				}
				Err(e) => {
					eprintln!("tcp_stream.read() error: {e:#?}");
					continue 'collect_request;
				}
			}

			if partial_request.is_finished() {
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

		println!("got request: {:#?}", request);

		tcp_stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap()
	}
}
