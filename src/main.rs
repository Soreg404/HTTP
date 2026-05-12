use http::message_common::{message_common_dbg, RequestCollector};

fn main() {
	let sample_normal = b"GET / HTTP/1.1\r\nHost: example.com\r\n\
	Accept: */*\r\ncontent-length: 5\r\n\r\nhello world!";

	test_request_collector(sample_normal);
}

const CYAN: &str = "\x1b[96m";
const C_NUL: &str = "\x1b[0m";


fn test_request_collector(sample: &[u8]) {
	println!("{CYAN}==== Test ===={C_NUL}");
	println!("request: ({} bytes)\n<<<<<{:.100}>>>>>", sample.len(),
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

	println!("is_finished: {:?}", rc.parse_result());

	println!("{CYAN}debug:{C_NUL}");
	message_common_dbg(&rc);
}
