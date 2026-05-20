use http::message_common::RequestCollector;

#[test]
fn collect_request_and_get_uri() {
	let mut rc = RequestCollector::new();

	let sample =
		b"GET /resource/entity/flying%20bison HTTP/1.1\r\n\
		host: localhost\r\n\
		\r\n";
	let n = rc.push_bytes(sample);
	assert_eq!(n, sample.len());
	assert!(rc.is_finished());

	let req = rc.
}
