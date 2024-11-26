use std::collections::HashMap;

#[derive(Default)]
pub struct Url {
	scheme: String,
	host: String,
	path: String,
	query: String,
	fragment: String,
}

fn single_hex_to_dec(a: char) -> Option<u8> {
	let an = (a as u8).wrapping_sub(b'0');
	if an <= 9 {
		return Some(an);
	}
	let al = (a as u8 & 0xdf).wrapping_sub(b'A');
	if al <= 5 {
		return Some(al + 10);
	}
	None
}
#[test]
fn single_hex_to_dec_test() {
	assert_eq!(single_hex_to_dec('1'), Some(1));
	assert_eq!(single_hex_to_dec('A'), Some(10));
	assert_eq!(single_hex_to_dec('d'), Some(13));
	assert_eq!(single_hex_to_dec('f'), Some(15));
	assert_eq!(single_hex_to_dec('g'), None);
}

fn single_dec_to_hex(c: u8) -> Option<char> {
	if c <= 9 {
		Some((c + b'0') as char)
	} else if c <= 15 {
		Some((c - 10 + b'A') as char)
	} else {
		None
	}
}
#[test]
fn single_dec_to_hex_test() {
	assert_eq!(single_dec_to_hex(0), Some('0'));
	assert_eq!(single_dec_to_hex(1), Some('1'));
	assert_eq!(single_dec_to_hex(14), Some('E'));
	assert_eq!(single_dec_to_hex(15), Some('F'));
	assert_eq!(single_dec_to_hex(16), None);
	assert_eq!(single_dec_to_hex(254), None);
	assert_eq!(single_dec_to_hex(255), None);
}

impl Url {
	pub fn unescape(text: impl AsRef<str>) -> String {
		let mut result_vec = Vec::<u8>::new();
		let mut text_iter = text.as_ref().chars();
		let mut in_escape = false;
		let mut escape_second_part = false;
		let mut escape_word: u8 = 0;
		let mut is_bad_escape = false;
		while let Some(c) = text_iter.next() {
			if in_escape {
				let hex_dig = single_hex_to_dec(c);
				if hex_dig.is_none() {
					is_bad_escape = true;
				}
				match escape_second_part {
					false => {
						escape_second_part = true;
						if !is_bad_escape {
							escape_word = 16 * hex_dig.unwrap();
						}
					}
					true => {
						in_escape = false;
						if !is_bad_escape {
							escape_word += hex_dig.unwrap();
							result_vec.push(escape_word);
						}
					}
				};
			} else if c == '%' {
				in_escape = true;
				escape_second_part = false;
				is_bad_escape = false;
			} else if c == '+' {
				result_vec.push(b' ');
			} else {
				result_vec.push(c as u8);
			}
		}
		let result_str = String::from_utf8_lossy(result_vec.as_slice()).to_string();
		result_str
	}
}
#[test]
fn url_unescape_test() {
	let str1 = "no percent encoding";
	assert_eq!(Url::unescape(str1), str1);

	assert_eq!(
		Url::unescape("percent%20encoding%20spaces"),
		"percent encoding spaces"
	);

	assert_eq!(
		Url::unescape("special+characters+\
		%21%23%24%26%27%28%29%2A%2B%2C%2F%3A%3B%3D%3F%40%5B%5D"),
		"special characters !#$&'()*+,/:;=?@[]"
	);

	assert_eq!(
		Url::unescape("utf-8%20%E7%8C%AB"),
		"utf-8 猫"
	);

	assert_eq!(Url::unescape("space+plus"), "space plus");

	assert_eq!(Url::unescape("bad+escape+removed: [%jk]"), "bad escape removed: []");

	assert_eq!(Url::unescape("invalid utf-8: %E7%8C"), "invalid utf-8: \u{FFFD}");
}

impl Url {
	pub fn escape(text: impl AsRef<str>) -> String {
		let text_bytes = text.as_ref().as_bytes();

		let mut ret = Vec::<u8>::with_capacity(text_bytes.len() * 2);
		for c in text_bytes {
			if c.is_ascii_alphanumeric() || match c {
				b'-' | b'_' | b'.' | b'~' => true,
				_ => false,
			} {
				ret.push(*c);
			} else if *c == b' ' {
				ret.push(b'+');
			} else {
				ret.push(b'%');
				ret.push(single_dec_to_hex(*c >> 4).unwrap() as u8);
				ret.push(single_dec_to_hex(*c & 0x0f).unwrap() as u8);
			}
		}
		String::from_utf8(ret).unwrap()
	}
}

#[test]
fn url_escape_test() {
	assert_eq!(Url::escape("pass-through"), "pass-through");
	assert_eq!(Url::escape("space space"), "space+space");
	assert_eq!(Url::escape("special chars: /=+"), "special+chars%3A+%2F%3D%2B");
}


impl Url {
	pub fn parse_query(query: &str) -> HashMap<String, String> {
		let mut ret = HashMap::<String, String>::new();
		let fields = query.split('&');
		for field in fields {
			let (key, val) = field.split_at(field.find('=')
				.expect("*temp* equal sign not found"));
			ret.insert(key.to_string(), val[1..].to_string());
		}
		ret
	}
}

#[test]
fn url_parse_query_test() {
	let eq = HashMap::<String, String>::from([
		("key".to_string(), "value".to_string())
	]);
	assert_eq!(Url::parse_query("key=value"), eq);
}
