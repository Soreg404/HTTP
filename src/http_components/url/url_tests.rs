use std::collections::HashMap;

use super::*;

#[cfg(test)]
mod unescape {
	use super::*;
	#[test]
	fn no_encoding() {
		let str = "no percent encoding";
		assert_eq!(Url::decode(str), str.as_bytes().to_vec());
	}

	#[test]
	fn spaces() {
		assert_eq!(
			Url::decode("percent%20encoding%20spaces"),
			"percent encoding spaces".as_bytes().to_vec()
		);
	}

	#[test]
	fn space_plus() {
		assert_eq!(Url::decode("space+plus"), b"space plus".to_vec());
	}

	#[test]
	fn special_chars() {
		assert_eq!(
		Url::decode("special+characters+\
		%21%23%24%26%27%28%29%2A%2B%2C%2F%3A%3B%3D%3F%40%5B%5D"),
		b"special characters !#$&'()*+,/:;=?@[]".to_vec(),
	);
	}

	#[test]
	fn utf() {
		assert_eq!(
			Url::decode("utf-8%20%E7%8C%AB"),
			"utf-8 猫".as_bytes().to_vec(),
		);
	}

	#[test]
	fn bad_escape() {
		assert_eq!(
			Url::decode("bad+escape+removed: [%jk]"),
			b"bad escape removed: []".to_vec()
		);
	}

	#[test]
	fn invalid_utf() {
		assert_eq!(
			Url::decode("invalid utf-8: %E7%8C"),
			b"invalid utf-8: \xE7\x8C".to_vec()
		);
	}
}

#[cfg(test)]
mod escape {
	use super::*;

	#[test]
	fn no_encoding() {
		assert_eq!(
			Url::encode(b"pass-through".to_vec().as_ref()),
			"pass-through");
	}
	#[test]
	fn space() {
		assert_eq!(
			Url::encode(b"space space".to_vec().as_ref()),
			"space+space");
	}
	#[test]
	fn special_chars() {
		assert_eq!(
			Url::encode("special chars: /=+".as_bytes().to_vec().as_ref()),
			"special+chars%3A+%2F%3D%2B");
	}
}

#[test]
fn parse_query_string() {
	let eq = HashMap::<Vec<u8>, Option<Vec<u8>>>::from([
		("key".as_bytes().to_vec(), Some("value".as_bytes().to_vec()))
	]);
	assert_eq!(Url::parse_query_string("key=value"), eq);
}
