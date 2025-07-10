use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::io::Read;
use std::str::FromStr;

#[cfg(test)]
#[path = "./url_tests.rs"]
mod url_tests;

#[derive(Default, Clone)]
pub struct Url {
	// [scheme://][domain][:port]/[path][?query_string][#fragment]
	pub scheme: Option<String>,
	pub domain: Option<String>,
	pub port: Option<u16>,
	pub path: String,
	pub query_string: String,
	pub fragment: String,
}

impl FromStr for Url {
	type Err = ();

	/** todo: this needs redoing */
	fn from_str(url_str: &str) -> Result<Self, Self::Err> {

		let (request, fragment_pos) = {
			let pos = url_str.find('#');
			if pos.is_some() {
				url_str.split_at(pos.unwrap())
			} else {
				(url_str, "")
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

		let path = path.strip_suffix('/').unwrap_or(path);

		Ok(Self {
			path: path.to_owned(),
			query_string: query_str.to_owned(),
			fragment: fragment_pos.to_owned(),
			..Self::default()
		})
	}
}

impl Url {
	pub fn decode(text: &str) -> Vec<u8> {
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
	pub fn encode(data: &[u8]) -> String {
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
				let field = Self::decode(field);
				ret.insert(field, None);
			} else {
				let (key, val) = field.split_at(eq_find.unwrap());
				ret.insert(Self::decode(key), Some(Self::decode(&val[1..])));
			}
		}
		ret
	}

	pub fn get_request_target(&self) -> String {
		let mut result = String::from("/");
		result.push_str(&self.path);
		if !self.query_string.is_empty() {
			result.push('?');
			result.push_str(&self.query_string);
		}
		result
	}

	pub fn from_absolute(line: &str) -> Url {
		// scheme://host/path
		unimplemented!()
	}
	pub fn from_relative(line: &str) -> Url {
		// path || /path
		// default host & scheme
		unimplemented!()
	}
}

impl Display for Url {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.scheme.is_some() {
			write!(f, "{}://", self.scheme.as_ref().unwrap())?;
		}
		if self.domain.is_some() {
			write!(f, "{}", self.domain.as_ref().unwrap())?;
		}
		if self.port.is_some() {
			write!(f, ":{}", self.port.as_ref().unwrap())?;
		}

		write!(f, "/{}", self.path.as_str())?;

		if !self.query_string.is_empty() {
			write!(f, "?{}", self.query_string.as_str())?;
		}
		if !self.fragment.is_empty() {
			write!(f, "?{}", self.fragment.as_str())?;
		}

		Ok(())
	}
}

impl Debug for Url {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"<URL:[{}]://[{}]:[{}]/{}[?{}][#{}]>",
			self.scheme.as_ref().unwrap_or_default(),
			self.domain.as_ref().unwrap_or_default(),
			self.port.unwrap_or_default(),
			self.path,
			self.query_string,
			self.fragment
		)
	}
}
