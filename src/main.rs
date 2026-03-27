use http::message_common::RequestCollector;

fn main() {
	let mut a = RequestCollector::new();

	let sample = b"GET / HTTP/1.1\r\nHost: example.com\r\nAccept: */*\r\n\r\nhello world!";
	dbg!(sample.len());

	let n = a.push_bytes(sample);

	dbg!(n);
	println!("bytes left: {:?}", String::from_utf8_lossy(&sample[n..]));
	dbg!(a.working_buffer_reader);
	dbg!(String::from_utf8_lossy(&a.working_buffer));
}
