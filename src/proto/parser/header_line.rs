use crate::proto::parser::ParseError;
use crate::proto::parser::ParseError::TBD;

#[derive(Debug, Eq, PartialEq)]
pub enum ParseResult<'a> {
	Empty,
	Ok {
		field_name: &'a [u8],
		field_value: &'a [u8],
	},
	Err(ParseError),
}

enum ParseState {
	FieldName,
	WhitespaceBeforeColon,
	WhitespaceAfterColon,
	FieldValue,
}

fn valid_first_byte_of_field_name(c: &u8) -> bool {
	c.is_ascii_alphanumeric()
		|| match c {
		// todo: what are the allowed characters here?
		_ => false
	}
}

fn valid_nth_byte_of_field_name(c: u8) -> bool {
	c.is_ascii_alphanumeric()
		|| match c {
		_ => false
	}
}

pub fn parse_header_line(line: &[u8]) -> ParseResult {
	use ParseState::*;
	use ParseResult::*;

	if line.is_empty() {
		return Empty;
	}

	if line
		.get(0)
		.map(valid_first_byte_of_field_name)
		!= Some(true) {
		return Err(TBD);
	}

	let mut state = FieldName;

	let mut filed_name_end_index = 0usize;
	let mut field_value_start_index = 0usize;
	let mut last_non_ws_index = 0usize;

	for (i, b) in line.iter()
		.copied().enumerate().skip(1) {
		if !b.is_ascii() {
			panic!("Non-ascii character in header line");
		}

		match state {
			FieldName => {
				if b == b' ' {
					state = WhitespaceBeforeColon;
					filed_name_end_index = i;
					continue;
				}

				if b == b':' {
					state = WhitespaceAfterColon;
					filed_name_end_index = i;
					continue;
				}

				if !valid_nth_byte_of_field_name(b) {
					return Err(TBD);
				}
			}
			WhitespaceBeforeColon => {
				if b == b':' {
					state = WhitespaceAfterColon;
				} else if b != b' ' {
					return Err(TBD)
				}
			}
			WhitespaceAfterColon => {
				if b.is_ascii_whitespace() {
					continue;
				} else {
					state = FieldValue;
					field_value_start_index = i;
				}
			}
			FieldValue => {
				if !b.is_ascii_whitespace() {
					last_non_ws_index = i;
				}
			}
		}
	}

	let field_name = &line[..filed_name_end_index];
	let field_value = &line[field_value_start_index..last_non_ws_index + 1];

	let trailing_whitespace = &line[last_non_ws_index + 1..];
	for c in trailing_whitespace.iter().copied() {
		if c != b' ' {
			return Err(TBD);
		}
	}

	if field_name.len() == 0 {
		return Err(TBD);
	}

	Ok {
		field_name,
		field_value,
	}
}


#[test]
fn test_parse_header_line() {
	use ParseResult::*;

	let healthy_header_line = b"host: unstd.pl";
	assert_eq!(
		parse_header_line(healthy_header_line),
		Ok {
			field_name: &healthy_header_line[..4],
			field_value: &healthy_header_line[6..]
		}
	);

	let leading_ws = b" host: unstd.pl";
	assert_eq!(
		parse_header_line(leading_ws),
		Err(TBD)
	);
	let no_colon = b"host unstd.pl";
	assert_eq!(
		parse_header_line(no_colon),
		Err(TBD)
	);
	let empty_field_name = b": unstd.pl";
	assert_eq!(
		parse_header_line(empty_field_name),
		Err(TBD)
	);
	let tab_character_at_eol = b"host: unstd.pl	";
	assert_eq!(
		parse_header_line(tab_character_at_eol),
		Err(TBD)
	);
}
