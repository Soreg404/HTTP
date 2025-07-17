use std::str::FromStr;
use crate::proto::internal::parser;
use crate::proto::mime_type::MimeType;
use crate::proto::parse_error::HTTPParseError::MalformedMessage;
use crate::proto::parse_error::{HTTPParseError, MalformedMessageKind};

#[derive(Debug, Eq, PartialEq, Default, Clone)]
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
	type Err = HTTPParseError;

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

#[derive(Debug, Eq, PartialEq)]
pub struct FormDataFields {
	pub name: String,
	pub filename: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum HTTPHeaderEnum {
	Other,
	ContentLength(usize),
	ContentType(MimeType),
	ContentDispositionFormData(FormDataFields),
}

impl HTTPHeaderEnum {
	pub fn from_header(header: &HTTPHeader) -> Result<Self, HTTPParseError> {
		match header.name.to_lowercase().as_str() {
			"content-length" => {
				match header.value.parse::<usize>() {
					Ok(v) => Ok(HTTPHeaderEnum::ContentLength(v)),
					Err(e) => Err(
						HTTPParseError::MalformedMessage(
							MalformedMessageKind::Other
						)
					)
				}
			}
			"content-type" => {
				let mut split_args = header.value.split(';');

				let media_type = split_args.next().unwrap_or_default();

				Ok(HTTPHeaderEnum::ContentType(
					match media_type {
						"text/plain" => MimeType::TextPlain,
						"text/html" => MimeType::TextHtml,
						"application/json" | "text/json" => MimeType::TextJson,
						"image/jpeg" => MimeType::ImageJpg,
						"image/png" => MimeType::ImagePng,
						"multipart/form-data" => {
							// todo: parse HTTP Header value parameters generally
							match split_args.next() {
								None => return Err(
									HTTPParseError::MalformedMessage(
										MalformedMessageKind::MimetypeMissingBoundaryParam
									)
								),
								Some(param) => {
									match param.find('=') {
										None => return Err(
											HTTPParseError::MalformedMessage(
												MalformedMessageKind
												::MimetypeParamMissingEqualSign
											)
										),
										Some(pos) => {
											MimeType::Multipart(
												param[pos + 1..].trim().to_string()
											)
										}
									}
								}
							}
						}
						_ => MimeType::Unspecified
					}
				))
			},
			"content-disposition" => {
				match parser::header_content_disposition_value(header.value.as_str()) {
					None => {
						Err(MalformedMessage(
							MalformedMessageKind::HeaderContentDisposition
						))
					}
					Some(v) => {
						Ok(HTTPHeaderEnum::ContentDispositionFormData(
							FormDataFields {
								name: v.name,
								filename: v.filename,
							}
						))
					}
				}
			}
			_ => Ok(HTTPHeaderEnum::Other),
		}
	}
}


#[cfg(test)]
mod tests {
	use crate::proto::mime_type::MimeType;
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

	#[test]
	fn match_content_type_multipart_header() {
		let header = HTTPHeader {
			name: String::from("content-type"),
			value: String::from("multipart/form-data; boundary=test")
		};

		assert_eq!(
			HTTPHeaderEnum::from_header(&header),
			Ok(HTTPHeaderEnum::ContentType(MimeType::Multipart(String::from("test")))),
			"simple boundary mime type param"
		);

	}
}
