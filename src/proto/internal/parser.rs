use std::str::FromStr;
use crate::proto::header::HTTPHeader;
use crate::proto::parse_error::{HTTPParseError, MalformedMessageKind};
use crate::proto::parse_error::HTTPParseError::{IllegalByte, MalformedMessage};
use crate::proto::Url;

pub struct FirstLineRequest {
	pub method: String,
	pub url: Url,
	pub version: String,
}

pub fn get_first_line_request(line: &[u8]) -> Result<FirstLineRequest, HTTPParseError> {
	let line = validate_http_line_bytes(line)?;

	let line_parts = line
		.split_whitespace()
		.map(|s| s.to_owned())
		.collect::<Vec<String>>();

	if line_parts.len() != 3 {
		return Err(MalformedMessage(MalformedMessageKind::FirstLine));
	}

	let method = line_parts.get(0).unwrap().to_owned();
	// todo: check method validity

	let url = match Url::from_str(line_parts.get(1).unwrap()) {
		Ok(v) => v,
		Err(e) => {
			return Err(MalformedMessage(MalformedMessageKind::UrlGeneral))
		}
	};

	let version = line_parts.get(2).unwrap().to_owned();
	// todo: check version

	Ok(FirstLineRequest {
		method,
		url,
		version,
	})
}

pub struct FirstLineResponse {
	pub version: String,
	pub status_code: u16,
	pub status_text: String,
}
pub fn get_first_line_response(line: &[u8]) -> Result<FirstLineResponse, HTTPParseError> {
	let line = validate_http_line_bytes(line)?;

	let line_parts = line
		.split_whitespace()
		.map(|s| s.to_owned())
		.collect::<Vec<String>>();

	if line_parts.len() < 3 {
		return Err(MalformedMessage(MalformedMessageKind::FirstLine));
	}

	let version = line_parts.get(0).unwrap().to_owned();
	// todo: check version

	let status_code = match line_parts.get(1).unwrap()
		.parse::<u16>() {
		Ok(v) => v,
		Err(e) => return Err(MalformedMessage(
			MalformedMessageKind::FirstLineStatusCode
		)),
	};
	// todo: check method validity

	let status_text = line_parts[2..].join(" ");

	Ok(FirstLineResponse {
		version,
		status_code,
		status_text,
	})
}


#[cfg(delete)]
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

fn validate_http_line_bytes(line_bytes: &[u8]) -> Result<&str, HTTPParseError> {
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

	Ok(
		HTTPHeader::from_str(line)
			.expect("from_str currently can't return Err, \
						if it does then some breaking changes with HTTPHeader happened,\
							 remember to fix here")
	)
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
