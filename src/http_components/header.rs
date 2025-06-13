use std::str::FromStr;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct HTTPHeader {
	pub name: String,
	pub value: String,
}

impl HTTPHeader {
	pub fn new(name: String, value: String) -> Self {
		Self {
			name,
			value,
		}
	}
}

impl FromStr for HTTPHeader {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(
			match s.find(':') {
				Some(index) => {
					let (name, value) = s.split_at(index);
					Self {
						name: name.trim().to_string(),
						value: value[1..].trim().to_string(),
					}
				}
				None => Self::default()
			}
		)
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_header_line() {
		assert_eq!(
			HTTPHeader::from_str(""),
			Ok(HTTPHeader::default()),
			"from empty str - should be Ok(empty) unless breaking change"
		);

		assert_eq!(
			HTTPHeader::from_str(":"),
			Ok(HTTPHeader::default()),
			"single semicolon, should output Ok(empty), no error handling for now"
		);

		assert_eq!(
			HTTPHeader::from_str("some-weird-value:"),
			Ok(HTTPHeader {
				name: String::from("some-weird-value"),
				value: String::from("")
			}),
			"edge case: badly formatted header with semicolon on end"
		);

		let valid_header =
			Ok(HTTPHeader {
				name: String::from("content-type"),
				value: String::from("555"),
			});

		assert_eq!(
			HTTPHeader::from_str("content-type: 555"),
			valid_header,
			"simple parse, correct line, should not error"
		);

		assert_eq!(
			HTTPHeader::from_str("     content-type   :      555    "),
			valid_header,
			"parse with excessive whitespace"
		);

		assert_eq!(
			HTTPHeader::from_str("content-type"),
			Ok(HTTPHeader::default()),
			"badly formatted header"
		);
	}
}
