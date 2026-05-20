use http::message_common::{message_common_dbg, RequestCollector};

fn main() {
	test_request_collector(
		"simple request",
		b"GET / HTTP/1.1\r\n\
		Host: example.com\r\n\
		Accept: */*\r\n\
		content-length: 5\r\n\
		\r\n\
		hello world!",
	);

	test_request_collector(
		"multipart",
		b"GET / HTTP/1.1\r\n\
		Host: example.com\r\n\
		content-length: 76\r\n\
		content-type: multipart/form-data; boundary=ABCD\r\n\
		\r\n\
		--ABCD\r\n\
		content-disposition: form-data; name=\"foo\"\r\n\
		\r\n\
		hello world!\r\n\
		--ABCD--",
	);
}

const C_MAG: &str = "\x1b[95m";
const CYAN: &str = "\x1b[96m";
const C_NUL: &str = "\x1b[0m";


fn test_request_collector(test_name: &str, sample: &[u8]) {
	println!("{C_MAG}===test: {test_name:?}{C_NUL}");
	println!("request: ({} bytes)\n{CYAN}<<<<<{C_NUL}{:.100}{CYAN}>>>>>{C_NUL}", sample.len(),
			 String::from_utf8_lossy(sample));
	println!("{CYAN}--------------{C_NUL}");

	let mut rc = RequestCollector::new();

	println!("{CYAN}first batch{C_NUL}");
	let bytes_read = rc.push_bytes(&sample[..sample.len() / 2]);
	println!("bytes read: {} of {}", bytes_read, sample.len() / 2);
	println!("{CYAN}second batch{C_NUL}");
	let bytes_read = rc.push_bytes(&sample[sample.len() / 2..]);
	println!("bytes read: {} of {}", bytes_read, sample.len() - sample.len() / 2);
	println!("leftover bytes: {:?}", String::from_utf8_lossy(
		&sample[sample.len() / 2 + bytes_read..]));

	println!("is_finished: {:?}", rc.is_finished());

	println!("{CYAN}debug:{C_NUL}");
	message_common_dbg(&rc);

	println!("{C_MAG}---ended: {test_name:?}{C_NUL}");
	print!("\n\n");
}
