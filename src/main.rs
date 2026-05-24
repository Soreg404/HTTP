use std::borrow::Cow;
use http::{ResponseBuilder, StatusCode, Version};
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::Path;

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

fn match_route(path_bytes: &[u8]) -> Result<ResponseBuilder, ()> {
	let mut buffer = Box::new([0u8; 0x4000]);
	let mut part_it = http::UrlPartsIterator::new(path_bytes)
		.decoded_to_buffer(buffer.as_mut_slice());

	Ok(match part_it.next() {
		None => ResponseBuilder {
			status_code: StatusCode::SUCCESS,
			status_description: "OK".to_string(),
			version: Version::HTTP_1_1,
			headers: vec![
				b"content-type: text/html".to_vec(),
			],
			body: b"<h1>main page</h1><h2>hello!</h2>".to_vec(),
		},
		Some(Ok(p)) => match p {
			b"hello" => ResponseBuilder {
				status_code: StatusCode::SUCCESS,
				status_description: "OK".to_string(),
				version: Version::HTTP_1_1,
				headers: vec![
					b"content-type: text/html".to_vec(),
				],
				body: include_bytes!("../local/m.txt").to_vec(),
			},
			b"img" => ResponseBuilder {
				status_code: StatusCode::SUCCESS,
				status_description: "OK".to_string(),
				version: Version::HTTP_1_1,
				headers: vec![
					b"content-type: image/jpg".to_vec(),
				],
				body: include_bytes!("../local/upload.jpg").to_vec(),
			},

			s if s == "cześć".as_bytes() => ResponseBuilder {
				status_code: StatusCode::SUCCESS,
				status_description: "DOBRZE".to_string(),
				version: Version::HTTP_1_1,
				headers: vec![b"content-type: text/html".to_vec()],
				body: b"<h1>Polish fucker detected</h1>".to_vec(),
			},

			b"serve" => match part_it.next() {
				None => ResponseBuilder::quick_404(),
				Some(Err(_n)) => return Err(()),
				Some(Ok(p)) => {
					let p = String::from_utf8_lossy(p);
					let mut dir = std::fs::read_dir(
						Path::new("local/serve")
					).unwrap();

					for entry in dir {
						let entry = match entry {
							Err(_e) => continue,
							Ok(v) => v
						};
						let path = entry.path();
						if !path.is_file() {
							continue;
						}
						if path.file_name() != Some(OsStr::new(p.as_ref())) {
							continue;
						}
						let ext = match path.extension() {
							Some(ext) => ext.to_string_lossy(),
							None => Cow::from("png")
						};
						let ct = {
							let mut tmp = b"content-type: image/".to_vec();
							tmp.extend_from_slice(ext.as_bytes());
							tmp
						};

						let body = std::fs::read(path).unwrap();

						return Ok(ResponseBuilder {
							status_code: StatusCode::SUCCESS,
							status_description: "OK".to_string(),
							version: Version::HTTP_1_1,
							headers: vec![
								ct
							],
							body,
						});
					}

					ResponseBuilder::quick_404()
				}
			}

			_ => ResponseBuilder::quick_404()
		},
		Some(Err(_n)) => return Err(())
	})
}
