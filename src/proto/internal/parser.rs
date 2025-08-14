use std::str::FromStr;
use MalformedMessageKind::{FirstLine, FirstLineStatusCode, MalformedHTTPVersion};
use crate::MalformedMessageKind::MalformedHeader;
use crate::proto::header::{HTTPHeader, HTTPHeaderRef};
use crate::proto::parse_error::{HTTPParseError, MalformedMessageKind};
use crate::proto::parse_error::HTTPParseError::{IllegalByte, MalformedMessage};

pub struct FirstLineRequest {
	pub method: String,
	pub target: String,
	pub version: (u8, u8),
}
impl TryFrom<&[u8]> for FirstLineRequest {
	type Error = HTTPParseError;
	fn try_from(line: &[u8]) -> Result<Self, Self::Error> {
		let line = validate_http_line_bytes(line)?;

		let line_parts = line
			.split_whitespace()
			.collect::<Vec<&str>>();

		if line_parts.len() != 3 {
			return Err(MalformedMessage(FirstLine));
		}

		let method = line_parts.get(0).unwrap().to_owned().to_string();
		// todo: check method validity

		// let url = match Url::from_str(line_parts.get(1).unwrap()) {
		// 	Ok(v) => v,
		// 	Err(e) => {
		// 		return Err(MalformedMessage(MalformedMessageKind::UrlGeneral))
		// 	}
		// };
		let target = line_parts.get(1).unwrap().to_owned().to_string();

		let version = line_parts.get(2).unwrap().to_owned();
		let version = http_version_from_str(version)?;
		// todo: check version

		Ok(FirstLineRequest {
			method,
			target,
			version,
		})
	}
}


pub struct FirstLineResponse {
	pub version: (u8, u8),
	pub status_code: u16,
	pub status_text: String,
}

impl TryFrom<&[u8]> for FirstLineResponse {
	type Error = HTTPParseError;

	fn try_from(line: &[u8]) -> Result<Self, Self::Error> {
		let line = validate_http_line_bytes(line)?;

		let line_parts = line
			.split_whitespace()
			.collect::<Vec<&str>>();

		if line_parts.len() < 3 {
			return Err(MalformedMessage(FirstLine));
		}

		let version = line_parts.get(0).unwrap();
		let version = http_version_from_str(version)?;

		let status_code = match line_parts.get(1).unwrap()
			.parse::<u16>() {
			Ok(v) => v,
			Err(e) => return Err(
				MalformedMessage(FirstLineStatusCode)),
		};

		let status_text = line_parts[2..].join(" ");

		Ok(FirstLineResponse {
			version,
			status_code,
			status_text,
		})
	}
}


#[cfg(feature = "delete")]
pub fn parse_request_first_line(line_bytes: &[u8]) -> Result<RequestFirstLine, HTTPParseError> {
	let line = validator::ascii_to_string(line_bytes)?;

	// todo: should accept multiple spaces btwn GET, URL and VER?
	let line_parts = line
		.split_whitespace()
		.map(|s| s.to_owned())
		.collect::<Vec<String>>();

	if line_parts.len() != 3 {
		Err(MalformedMessage(Other))
	} else {
		let method = line_parts.get(0).unwrap().to_owned();
		// todo: check method validity

		let url = match Url::from_str(line_parts.get(1).unwrap()) {
			Ok(v) => v,
			Err(e) => {
				return Err(MalformedMessage(Other))
			}
		};

		let version = line_parts.get(2).unwrap().to_owned();
		// todo: check version

		Ok(RequestFirstLine {
			method,
			url,
			version,
		})
	}
}

pub fn validate_http_line_bytes(line_bytes: &[u8]) -> Result<&str, HTTPParseError> {
	for b in line_bytes.iter().cloned() {
		if !b.is_ascii_graphic() && b != b' ' {
			return Err(IllegalByte);
		}
	}
	Ok(
		unsafe { str::from_utf8_unchecked(line_bytes) }
	)
}

pub fn header_from_line_bytes(line_bytes: &[u8]) -> Result<HTTPHeader, HTTPParseError> {
	let line = validate_http_line_bytes(line_bytes)?;
	HTTPHeader::from_str(line)
}


#[derive(Debug)]
pub struct ContentDispositionHeaderValuesPartial {
	pub name: String,
	pub filename: Option<String>,
}
pub fn header_content_disposition_value(value: &str)
										-> Option<ContentDispositionHeaderValuesPartial> {
	let mut name = String::new();
	let mut filename: Option<String> = None;

	let mut splits = value.split(';');

	// bail if first part is not form-data
	match splits.next() {
		None => return None,
		Some(first_part) if first_part != "form-data" => return None,
		_ => {}
	};

	while let Some(key_val_part) = splits.next() {
		let (key, value) = match key_val_part.find('=') {
			None => (key_val_part, ""),
			Some(index) => key_val_part.split_at(index)
		};

		let value = {
			let no_eq_sign = value[1..].trim();
			let no_starting_quote = if no_eq_sign.starts_with('"') {
				&no_eq_sign[1..]
			} else {
				no_eq_sign
			};
			if no_starting_quote.ends_with('"') {
				&no_starting_quote[..no_starting_quote.len() - 1]
			} else {
				no_starting_quote
			}
		};

		match key.trim() {
			"name" => name = value.to_string(),
			"filename" => filename = Some(value.to_string()),
			_ => continue
		};
	}

	Some(
		ContentDispositionHeaderValuesPartial {
			name,
			filename,
		}
	)
}


fn http_version_from_str(s: &str) -> Result<(u8, u8), HTTPParseError> {
	let ver_proper = match s.strip_prefix("HTTP/") {
		None => return Err(MalformedMessage(MalformedHTTPVersion)),
		Some(d) => d
	};

	let bytes = ver_proper.as_bytes();
	if bytes.len() != 3 || bytes[1] != b'.' {
		return Err(MalformedMessage(MalformedHTTPVersion));
	}

	let ver_hi = match bytes.get(0).unwrap()
		.checked_sub(b'0') {
		None | Some(0) | Some(4..) => {
			return Err(MalformedMessage(MalformedHTTPVersion))
		}
		Some(v) => v
	};

	let ver_lo = match bytes.get(2).unwrap()
		.checked_sub(b'0') {
		None | Some(10..) => {
			return Err(MalformedMessage(MalformedHTTPVersion))
		}
		Some(v) => v
	};

	Ok((ver_hi, ver_lo))
}

#[test]
fn test_http_version_from_str() {
	assert_eq!(http_version_from_str("HTTP/1.1"), Ok((1, 1)));
	assert_eq!(http_version_from_str("HTTP/2.1"), Ok((2, 1)));
	assert_eq!(http_version_from_str("HTTP/3.0"), Ok((3, 0)));

	let e = Err(MalformedMessage(MalformedHTTPVersion));
	assert_eq!(http_version_from_str("HTTP/5.0"), e);
	assert_eq!(http_version_from_str("HTTP/0.12"), e);
	assert_eq!(http_version_from_str("HTTP/123"), e);
}
