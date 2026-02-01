use std::str::FromStr;
use crate::consts::{Method, Version};
use crate::proto::parser::ParseError;
use crate::proto::parser::ParseError::{FirstLine, TBD};

pub struct RequestFirstLine {
	pub method: Method,
	// todo: change to slice of line
	pub url_slice: Vec<u8>,
	pub version: Version,
}

// todo: FIXMEPLS
pub fn parse_request_first_line(line: &[u8]) -> Result<RequestFirstLine, ParseError> {
	assert_eq!(line.ends_with(b"\n"), false);

	// todo: "first draft"; fixmepls

	let s = String::from_utf8_lossy(line);
	let mut it = s.split_whitespace();

	let method;
	let url_slice;
	let version;

	match it.next() {
		None => return Err(FirstLine),
		Some(method_str) => {
			method = match Method::from_str(method_str) {
				Ok(v) => v,
				Err(e) => return Err(FirstLine)
			};

			match it.next() {
				None => return Err(FirstLine),
				Some(url_str) => {
					url_slice = url_str.as_bytes().to_owned();

					match it.next() {
						None => return Err(FirstLine),
						Some(version_str) => {
							version = match Version::from_str(version_str) {
								Ok(v) => v,
								Err(e) => return Err(FirstLine)
							};
						}
					}

				}
			}
		}
	}

	Ok(RequestFirstLine {
		method,
		url_slice,
		version,
	})

}
