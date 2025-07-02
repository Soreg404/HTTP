use crate::http_components::parse_error::HTTPParseError;
use crate::http_components::parse_error::HTTPParseError::IllegalByte;

pub trait IsValidHTTPByte {
	fn is_valid_http(&self) -> bool;
}

impl IsValidHTTPByte for u8 {
	fn is_valid_http(&self) -> bool {
		// todo: to be improved, very basic
		self >= &0x20 && self < &0x7f
	}
}


pub fn ascii_to_string(bytes: &[u8]) -> Result<String, HTTPParseError> {
	let mut ret = String::with_capacity(bytes.len());

	for b in bytes {
		if !b.is_valid_http() {
			// todo: error checking, invalid byte
			return Err(IllegalByte);
		}
		ret.push(*b as char);
	}
	Ok(ret)
}
