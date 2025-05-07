use std::collections::BTreeSet;
use crate::HTTPHeader;

#[derive(Default)]
pub struct HTTPHeaders {
	all_headers_raw: Vec<String>,
	headers: BTreeSet<HTTPHeader>,
}

impl HTTPHeaders {
	pub fn from_line_str(line: &str) {
		unimplemented!()
	}
}
