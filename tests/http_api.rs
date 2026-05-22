use http::RequestCollector;

#[test]
fn collect_request_and_get_uri() {

	let mut rc = RequestCollector::new();

	let sample =
		b"GET /resource/entity/flying%20bison HTTP/1.1\r\n\
		host: localhost\r\n\
		\r\n";
	rc.push_bytes(sample);
	match rc.is_finished() {
		Some(Ok(())) => {}
		_ => panic!(),
	};

	// let req = rc.
}
