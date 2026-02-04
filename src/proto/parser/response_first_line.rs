use std::num::ParseIntError;
use std::str::FromStr;
use crate::consts::{StatusCode, Version};
use crate::proto::parser::ParseError;
use crate::proto::parser::ParseError::FirstLine;

pub struct ResponseFirstLine {
	pub version: Version,
	pub status_code: StatusCode,
	pub status_desc: String,
}

// todo: ALSO FIXMEPLS
pub fn parse_response_first_line(line: &[u8])
								 -> Result<ResponseFirstLine, ParseError> {

	// todo: this is terrible

	let s = String::from_utf8_lossy(line);
	let mut it = s.split_whitespace();

	let version;
	let status_code;
	let status_desc;

	match it.next() {
		None => return Err(FirstLine),
		Some(v) => {
			version = Version::from_str(v)?;

			match it.next() {
				None => return Err(FirstLine),
				Some(v) => {
					status_code = StatusCode::try_from(
						match v.parse::<u32>() {
							Ok(code) => code,
							Err(_e) => return Err(ParseError::InvalidStatusCode),
						}
					)?;

					status_desc = it
						.map(|s| format!("{s} "))
						.collect::<String>();
				}
			}
		}
	}

	Ok(ResponseFirstLine {
		version,
		status_code,
		status_desc,
	})
}
