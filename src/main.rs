use http::{StatusCode, Version};
use std::io::{Read, Write};

#[path = "../samples.rs"]
mod samples;

fn main() {
	run_server();
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

		let req = match rc.finish() {
			None => continue,
			Some(v) => v
		};

		println!("finished request: {:?}", req);


		let rb = match match_route(
			&req
				.get_url_path_raw()[1..]
		) {
			Err(()) => {
				println!("failed to create response");
				continue;
			}
			Ok(v) => v,
		};

		stream.write(&rb.to_bytes()).unwrap();
	}
}

fn match_route(path_bytes: &[u8]) -> Result<http::ResponseBuilder, ()> {
	let mut part_it = path_bytes
		.split(|c| *c == b'/')
		.filter(|part| !part.is_empty())
		.map(|part| {
			http::url_decode_to_vec(part)
		});

	Ok(match part_it.next() {
		None => http::ResponseBuilder{
			status_code: StatusCode::SUCCESS,
			status_description: "OK".to_string(),
			version: Version::HTTP_1_1,
			headers: vec![
				b"content-type: text/html".to_vec(),
			],
			body: b"<h1>main page</h1><h2>hello!</h2>".to_vec(),
		},
		Some(p) => match p.as_slice() {
			b"hello" => http::ResponseBuilder {
					status_code: StatusCode::SUCCESS,
					status_description: "OK".to_string(),
					version: Version::HTTP_1_1,
					headers: vec![
						b"content-type: text/html".to_vec(),
					],
					body: include_bytes!("../local/m.txt").to_vec(),
				},
			b"img" => http::ResponseBuilder {
				status_code: StatusCode::SUCCESS,
				status_description: "OK".to_string(),
				version: Version::HTTP_1_1,
				headers: vec![
					b"content-type: image/jpg".to_vec(),
				],
				body: include_bytes!("../local/upload.jpg").to_vec(),
			},
			_ => http::ResponseBuilder::quick_404(),
		}
	})
}
