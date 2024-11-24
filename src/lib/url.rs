#[derive(Default)]
pub struct URL {
	scheme: String,
	host: String,
	path: String,
	query: String,
	fragment: String,
}

impl URL {

	pub fn unescape(text: impl AsRef<str>) {
		let mut result_vec = Vec::<u8>::new();
		let mut text_iter = text.as_ref().chars();
		for c in text_iter.next() {
			if c == '%' {
				let mut escape:[Option<char>; 2] = [None, None];
				escape[0] = text_iter.next();
				escape[1] = text_iter.next();
				if escape[0].is_none() || escape[1].is_none() {
					panic!("invalid escape");
				}
				let mut escape_str = [escape[0].unwrap() as u8, escape[1].unwrap() as u8];
				if !escape_str[0].is_ascii_alphanumeric() || !escape_str[1].is_ascii_alphanumeric() {
					panic!("invalid escape");
				}
				let mut escape_byte = escape_str[0] * 16 + escape_str[1];
				result_vec.push(escape_byte);
			} else {
				result_vec.push(c as u8);
			}
		}
		let result_str = String::from_utf8(result_vec).expect("invalid utf-8");
		println!("escaped: {}", result_str);
	}

}
