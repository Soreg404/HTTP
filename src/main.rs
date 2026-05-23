use std::io::{Read, Write};

#[path = "../samples.rs"]
mod samples;

fn main() {

	run_server();

	let mut rc = http::RequestCollector::new();
	rc.push_bytes(samples::MINIMAL);

	println!("finished request: {:?}", rc.finish());

}

fn run_server() {
	let tcp = std::net::TcpListener::bind("0.0.0.0:8500").unwrap();

	let mut buf = [0u8; 0x100];

	loop {
		let (mut stream, peer) = tcp.accept().unwrap();

		let mut rc = http::RequestCollector::new();

		while rc.parse_result().is_none() {
			let n = stream.read(&mut buf).unwrap();
			rc.push_bytes(&buf[..n]);
		}

		println!("got finished request, status: {:?}", rc.parse_result());
		println!("finished request: {:?}", rc.finish());

		stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
	}
}
