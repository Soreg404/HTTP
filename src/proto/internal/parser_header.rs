use std::str::FromStr;
use crate::HTTPParseError;
use crate::proto::internal::ascii_str::HTTPAsciiStr;

pub struct ContentDisposition<'header> {
	name: &'header str,
	filename: Option<&'header str>,
}


impl TryFrom<&str> for ContentDisposition<'_> {
	type Error = HTTPParseError;
	fn try_from(header_value: &str) -> Result<Self, Self::Error> {
		let split = {
			match header_value.find(';') {
				Some(index) => {
					let tmp = header_value.split_at(index);
					(tmp.0.trim(), tmp.1[1..].trim())
				},
				None => (header_value.trim(), ""),
			}
		};

		Ok(Self { name: "", filename: None })

	}
}

impl TryFrom<&HTTPAsciiStr<'_>> for ContentDisposition<'_> {
	type Error = HTTPParseError;
	fn try_from(header_value: &HTTPAsciiStr) -> Result<Self, Self::Error> {
		let split = {
			match header_value.find(';') {
				Some(index) => {
					let tmp = header_value.split_at(index);
					(tmp.0.trim(), tmp.1[1..].trim())
				},
				None => (header_value.trim(), ""),
			}
		};

		Ok(Self { name: "", filename: None })

	}
}
