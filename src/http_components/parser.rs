use std::str::FromStr;
use crate::http_components::parse_error::HTTPParseError;
use crate::http_components::parse_error::HTTPParseError::{IllegalByte, MalformedMessage};
use crate::http_components::validator::ascii_to_string;
use crate::{HTTPHeader, Url};
use crate::http_components::parse_error::MalformedMessageKind::Other;
use crate::http_components::validator;

pub struct FirstLinePartial {
	pub(crate) method: String,
	pub(crate) url: Url,
	pub(crate) version: String,
}

pub(crate) fn parse_request_first_line(line_bytes: &[u8]) -> Result<FirstLinePartial, HTTPParseError> {
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

		Ok(FirstLinePartial {
			method,
			url,
			version,
		})
	}
}

pub fn header_from_line(line_bytes: &[u8]) -> Result<HTTPHeader, HTTPParseError> {
	// maybe better to check_line_validity before to_string?

	let current_header_line_string = ascii_to_string(&line_bytes)?;

	Ok(
		HTTPHeader::from_str(
			current_header_line_string.as_str()
		)
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
