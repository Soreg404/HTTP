use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;

#[cfg(test)]
#[path = "./url_tests.rs"]
mod url_tests;

#[derive(Default)]
pub struct Url {
	// [scheme]://[Domain][Port]/[path]?[queryString]#[fragmentId]
	pub scheme: String,
	pub domain: String,
	pub port: u16,
	pub path: String,
	pub path_parts: Vec<Vec<u8>>,
	pub query: String,
	pub query_variables: HashMap<Vec<u8>, Option<Vec<u8>>>,
	pub fragment_id: String,
}

impl Url {
	pub fn from_request_str(request: &str) -> Self {
		let (request, fragment_id) = {
			let pos = request.find('#');
			if pos.is_some() {
				request.split_at(pos.unwrap())
			} else {
				(request, "")
			}
		};

		let (path, query_str) = {
			let pos = request.find('?');
			if pos.is_some() {
				let pair= request.split_at(pos.unwrap());
				(pair.0, &pair.1[1..])
			} else {
				(request, "")
			}
		};

		let path_parts = path.split('/')
			.filter(|s| !s.is_empty())
			.map(|s| Self::unescape(s))
			.collect();

		Self {
			path: path.to_owned(),
			path_parts,
			query: query_str.to_owned(),
			query_variables: Self::parse_query_string(query_str),
			fragment_id: fragment_id.to_owned(),
			..Self::default()
		}
	}
}

impl Url {
	pub fn unescape(text: &str) -> Vec<u8> {
		let mut result_vec = Vec::<u8>::new();
		let mut in_escape = false;
		let mut escape_second_char = false;
		let mut escape_word: Option<u8> = Some(0);
		for c in text.chars() {
			if in_escape {
				let hexit = match c {
					'0'..='9' => c as u8 - b'0',
					'a'..='f' => c as u8 - b'a' + 10,
					'A'..='F' => c as u8 - b'A' + 10,
					_ => {
						escape_word = None;
						0
					}
				};

				if escape_second_char {
					if escape_word.is_some() {
						result_vec.push(escape_word.unwrap() + hexit);
					}
					in_escape = false;
					escape_second_char = false;
				} else {
					if escape_word.is_some() {
						escape_word = Some(hexit * 16);
					}
					escape_second_char = true;
				}
			} else if c == '%' {
				in_escape = true;
				escape_word = Some(0);
			} else if c == '+' {
				result_vec.push(b' ');
			} else {
				result_vec.push(c as u8);
			}
		}
		result_vec
	}
	pub fn escape(data: &Vec<u8>) -> String {
		// TODO: new lines encoded as CRLF
		let mut ret = String::with_capacity((data.len() as f32 * 1.6) as usize);

		for c in data.iter().cloned() {
			if (c as char).is_ascii_alphanumeric() || match c {
				b'-' | b'_' | b'.' | b'~' => true,
				_ => false,
			} {
				ret.push(c as char);
			} else if c == b' ' {
				ret.push('+');
			} else {
				ret.push('%');
				ret.push_str(format!("{c:X}").as_ref());
			}
		}

		ret
	}
	pub fn parse_query_string(query: &str) -> HashMap<Vec<u8>, Option<Vec<u8>>> {
		let mut ret = HashMap::<Vec<u8>, Option<Vec<u8>>>::new();
		let fields = query.split('&');
		for field in fields {
			let eq_find = field.find('=');
			if eq_find.is_none() {
				let field = Self::unescape(field);
				ret.insert(field, None);
			} else {
				let (key, val) = field.split_at(eq_find.unwrap());
				ret.insert(Self::unescape(key), Some(Self::unescape(&val[1..])));
			}
		}
		ret
	}
	pub fn from_absolute(line: &str) -> Url {
		// scheme://host/path
		panic!("not implemented");
	}
	pub fn from_relative(line: &str) -> Url {
		// path || /path
		// default host & scheme
		panic!("not implemented");
	}
}

impl Display for Url {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!()
	}
}
