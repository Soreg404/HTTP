use http::{StatusCode, Version};
use std::io::{Read, Write};

#[path = "../samples.rs"]
mod samples;

macro_rules! calculate {
    // The pattern for a single `eval`
    (eval $e:expr) => {
        {
            let val: usize = $e; // Force types to be integers
            println!("{} = {}", stringify!{$e}, val);
        }
    };

    // Decompose multiple `eval`s recursively
    (eval $e:expr, $(eval $es:expr),+) => {{
        calculate! { eval $e }
        calculate! { $(eval $es),+ }
    }};
}

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

		// stream.write(&rb.to_bytes()).unwrap();
	}
}
