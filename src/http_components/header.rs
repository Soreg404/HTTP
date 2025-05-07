use crate::MimeType;

#[derive(PartialEq, Ord, PartialOrd, Eq)]
pub enum HTTPHeader {
	ContentLength(usize),
	ContentType(MimeType),
	Other(String, String),
	InvalidHeader(String, String),
	InvalidLine(String),
}


impl HTTPHeader {
	pub fn from_line(line: &str) -> Self {
		let split = line.find(":");
		if split.is_none() {
			return HTTPHeader::InvalidLine(line.to_string());
		}
		let (name, value) = line.split_at(split.unwrap());

		return Self::from_str(name, value);
	}

	pub fn from_str(name: &str, value: &str) -> Self {
		let name = name.trim();
		let value = value.trim();

		let name_lower = name.to_lowercase();

		match name_lower.as_str() {
			"content-length" => {
				match value.parse::<usize>() {
					Ok(v) => HTTPHeader::ContentLength(v),
					Err(_) => HTTPHeader::InvalidHeader()
				}
			}
		}
	}
	pub fn parse_value(&self) -> Vec<(String, Vec<(String, String)>)> {
		unimplemented!();

		// let mut in_quote = false;
		// let parts = self.value.split(|c| {
		// 	if c == '"' {
		// 		in_quote = !in_quote;
		// 	}
		// 	if in_quote { return false; }
		// 	if c == ',' {
		// 		true
		// 	} else {
		// 		false
		// 	}
		// });
		//
		// let mut ret: Vec<Vec<String>> = Vec::new();
		// for part in parts {
		// 	ret.push()
		// }

	}
}
