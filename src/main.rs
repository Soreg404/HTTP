use http::request_collector::RequestCollector;

fn main() {
	let mut rc = RequestCollector::new();

	let sample = b"GET /resource/entity/flying%20bison HTTP/1.1\r\n\
		host: localhost\r\n\
		content-length: 112\r\n\
		content-type: multipart/form-data; boundary=abc\r\n\
		\r\n\
		skip text\r\n\
		--abc\r\n\
		content-disposition: form-data; name=\"tf\"\r\n\
		content-type: text/plain\r\n\
		\r\n\
		hello worl!!\r\n\
		--abc--\r\n";

	rc.push_bytes(sample);
	match rc.is_finished() {
		Some(Ok(())) => {}
		_ => panic!(),
	};

	// let req = rc.
}
