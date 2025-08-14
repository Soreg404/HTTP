use crate::proto::parse_error::HTTPParseError::MalformedMessage;
use crate::proto::parse_error::{HTTPParseError, MalformedMessageKind};
use std::str::FromStr;
use MalformedMessageKind::MalformedHeader;

// todo: avoid self-referential and avoid creating new Strings for headers in partial message
#[derive(Debug, Eq, PartialEq)]
pub(super) struct HTTPHeaderRef<'a> {
	pub name: &'a str,
	pub value: &'a str,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct HTTPHeader {
	pub name: String,
	pub value: String,
}

impl HTTPHeader {
	pub fn new(name: &str, value: &str) -> Self {
		Self {
			name: name.to_string(),
			value: value.to_string()
		}
	}
}

impl FromStr for HTTPHeader {
	type Err = HTTPParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.find(':') {
			Some(index) => {
				let (name, value) = s.split_at(index);
				let tmp = Self {
					name: name.trim().to_string(),
					value: value[1..].trim().to_string(),
				};
				if tmp.name.is_empty() || tmp.value.is_empty() {
					Err(MalformedMessage(MalformedHeader))
				} else {
					Ok(tmp)
				}
			}
			None => Err(MalformedMessage(MalformedHeader))
		}
	}
}

impl Into<HTTPHeader> for HTTPHeaderRef<'_> {
	fn into(self) -> HTTPHeader {
		HTTPHeader {
			name: self.name.to_owned(),
			value: self.value.to_owned(),
		}
	}
}


// todo: parse content-type header into struct ContentType or smth

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_header_line() {
		assert_eq!(
			HTTPHeader::from_str(""),
			Err(MalformedMessage(MalformedHeader))
		);

		assert_eq!(
			HTTPHeader::from_str(":"),
			Err(MalformedMessage(MalformedHeader))
		);

		assert_eq!(
			HTTPHeader::from_str("some-weird-value:"),
			Err(MalformedMessage(MalformedHeader)),
		);

		let valid_header =
			Ok(HTTPHeader {
				name: "content-type".to_string(),
				value: "555".to_string(),
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
			Err(MalformedMessage(MalformedHeader)),
			"badly formatted header"
		);
	}
}
