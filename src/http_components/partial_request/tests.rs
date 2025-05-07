use crate::HTTPPartialRequest;

use super::*;

mod parse_header_line {
	use super::*;
	#[test]
	fn simple() {
		let header = HTTPPartialRequest
		::parse_header_line("content-type: text/plain");
		assert!(header.is_some());
		let header = header.unwrap();
		assert_eq!(header.name.as_str(), "content-type");
		assert_eq!(header.value.as_str(), "text/plain");
	}

	fn incomplete() {
		let header = HTTPPartialRequest
		::parse_header_line("content-type");
		assert!(header.is_none());
	}

	fn whitespaces() {
		let header = HTTPPartialRequest
		::parse_header_line("    content-type    :    text/plain     ");
		assert!(header.is_some());
		let header = header.unwrap();
		assert_eq!(header.name.as_str(), "content-type");
		assert_eq!(header.value.as_str(), "text/plain");
	}
}
