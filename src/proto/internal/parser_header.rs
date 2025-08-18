use crate::proto::internal::ascii_str::HTTPAsciiStr;
use crate::HTTPParseError;
use crate::HTTPParseError::MalformedMessage;
use crate::MalformedMessageKind::InvalidContentDisposition;

#[derive(Debug, PartialEq)]
pub struct ContentDisposition<'header> {
	usage: &'header str,
	name: &'header str,
	filename: Option<&'header str>,
}


fn split_on(s: &str, on: char) -> (&str, &str) {
	match s.find(on) {
		Some(index) => {
			let tmp = s.split_at(index);
			(tmp.0.trim(), tmp.1[1..].trim())
		}
		None => (s.trim(), ""),
	}
}

fn strip_quotes(s: &str) -> &str {
	s.strip_prefix('"').unwrap_or(s)
		.strip_suffix('"').unwrap_or(s)
}

// todo: make it TryFrom<HTTPHeader>
impl<'a> TryFrom<&'a str> for ContentDisposition<'a> {
	type Error = HTTPParseError;
	fn try_from(header_value: &'a str) -> Result<Self, Self::Error> {
		let split = split_on(header_value, ';');

		match split.0.to_lowercase().as_str() {
			"inline" | "attachment" | "form-data" => {}
			_ => return Err(MalformedMessage(InvalidContentDisposition))
		}

		let usage = split.0;

		// todo: http header values parser

		let mut name = "";
		let mut filename = None;

		let fields = split_on(split.1, ';');

		let mut assign = |part| -> Result<(), HTTPParseError> {
			let (key, value) = {
				let tmp = split_on(part, '=');
				(tmp.0.to_lowercase(), strip_quotes(tmp.1))
			};
			match key.as_str() {
				"name" => name = value,
				"filename" => filename = Some(value),
				_ => return Err(MalformedMessage(InvalidContentDisposition))
			}
			Ok(())
		};
		assign(fields.0)?;
		assign(fields.1)?;

		Ok(Self { usage, name, filename })
	}
}

#[test]
fn parse_content_disposition() {
	assert_eq!(
		ContentDisposition::try_from(
			"form-data; name=\"hello\"; filename=\"file.txt\""
		),
		Ok(ContentDisposition {
			usage: "form-data",
			name: "hello",
			filename: Some("file.txt"),
		})
	);
}

/*impl TryFrom<&HTTPAsciiStr<'_>> for ContentDisposition<'_> {
	type Error = HTTPParseError;
	fn try_from(header_value: &HTTPAsciiStr) -> Result<Self, Self::Error> {
		let split = {
			match header_value.find(';') {
				Some(index) => {
					let tmp = header_value.split_at(index);
					(tmp.0.trim(), tmp.1[1..].trim())
				}
				None => (header_value.trim(), ""),
			}
		};

		Ok(Self { name: "", filename: None })
	}
}*/


#[derive(Debug, PartialEq)]
pub struct ContentType<'a> {
	pub media_type: &'a str,
	pub boundary: Option<&'a str>,
}

// todo: needs improvement
impl<'a> TryFrom<&'a str> for ContentType<'a> {
	type Error = HTTPParseError;
	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		let (media_type, boundary) = split_on(value, ';');

		let boundary = split_on(boundary, '=').1;

		if media_type.eq_ignore_ascii_case("multipart/form-data") {
			Ok(ContentType {
				media_type,
				boundary: Some(boundary),
			})
		} else {
			Ok(ContentType {
				media_type,
				boundary: None,
			})
		}
	}
}

#[test]
fn parse_content_type() {
	assert_eq!(
		ContentType::try_from(
			"multipart/form-data; boundary=--hello"
		),
		Ok(
			ContentType {
				media_type: "multipart/form-data",
				boundary: Some("--hello"),
			}
		)
	)
}
