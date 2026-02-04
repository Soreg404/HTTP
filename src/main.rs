use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
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

		// drain_tcp(&mut tcp_stream);
		// tcp_stream.shutdown(Shutdown::Read).unwrap();


		let mut builder = http::response::Builder::new();
		builder.set_status(StatusCode::IM_A_TEAPOT);
		builder.push_header("helo", "world");

		let bytes = builder
			.into_response()
			.into_bytes();

		// tcp_stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap()
		tcp_stream.write_all(bytes.as_slice()).unwrap();


		println!("collecting response");
		let mut partial_response = http::response::Collector::new();

		'collect_response: loop {
			let mut buff = [0; 0xff];
			match tcp_stream.read(&mut buff) {
				Ok(n) => {
					if n == 0 {
						continue 'accept_connection;
					}

					println!("pushing bytes: {:?}", String::from_utf8_lossy(&buff[..n]));

					let n = partial_response.push_bytes(&buff[..n]);

					println!("consumed {n} bytes");
				}
				Err(e) => {
					eprintln!("tcp_stream.read() error: {e:#?}");
					continue 'collect_response;
				}
			}

			if partial_response.is_finished() {
				println!("is finished");
				break 'collect_response;
			}
		}

		let response = match partial_response.into_response() {
			Ok(v) => v,
			Err(e) => {
				eprintln!("http response parse error: {e:#?}");
				continue 'accept_connection;
			}
		};

		dbg!(response);
	}
}

fn drain_tcp(tcp_stream: &mut TcpStream) {
	// could be also a blocking read with timeout
	tcp_stream.set_nonblocking(true).unwrap();
	loop {
		let mut buff = [0; 0xff];
		match tcp_stream.read(&mut buff) {
			Ok(n) => {
				if n == 0 {
					break;
				}
				println!("connection drained bytes: {:?}",
						 String::from_utf8_lossy(&buff[..n]));
			}
			Err(e) if e.kind() == ErrorKind::WouldBlock => break,
			Err(e) => panic!("Other IO error: {e}"),
		}
	}

}
