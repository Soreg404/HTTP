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
		match line.find(":") {
			Some(index) => {
				let (name, value) = line.split_at(index);
				Self::from_name_value(name, value)
			}
			None => {
				HTTPHeader::InvalidLine(line.to_string())
			}
		}
	}

	pub fn from_name_value(name: &str, value: &str) -> Self {
		let name = name.trim().to_string();
		let value = value.trim().to_string();

		let name_lower = name.to_lowercase();

		match name_lower.as_str() {
			"content-length" => {
				match value.parse::<usize>() {
					Ok(v) => HTTPHeader::ContentLength(v),
					Err(_) => HTTPHeader::InvalidHeader(name, value)
				}
			}
			_ => HTTPHeader::Other(name, value)
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
