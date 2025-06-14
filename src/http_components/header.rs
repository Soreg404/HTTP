use std::str::FromStr;
use crate::MimeType;

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


/**
 * temporary, until no better solution
 */
impl HTTPHeader {
	pub fn parse_content_type_value(value: &str) -> MimeType {
		let mut splits = value.split(';');

		let first_part = match splits.next() {
			None => {
				return MimeType::Unspecified;
			}
			Some(first_part) => {
				first_part.trim().to_lowercase()
			}
		};

		if first_part != "multipart/form-data" {
			return match first_part.as_str() {
				"text/plain" => MimeType::TextPlain,
				"text/html" => MimeType::TextHtml,
				"application/json" | "text/json" => MimeType::TextJson,
				"image/jpeg" => MimeType::ImageJpg,
				"image/png" => MimeType::ImagePng,
				_ => MimeType::Unspecified
			};
		}

		let boundary_arg = match splits.next() {
			None => return MimeType::Unspecified,
			Some(s) => s.trim().to_string()
		};

		let boundary_value = match boundary_arg.split('=').nth(1) {
			None => return MimeType::Unspecified,
			Some(v) => v.to_string()
		};

		MimeType::Multipart(boundary_value)
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
