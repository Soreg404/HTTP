use crate::proto::rdx::Rdx;

#[derive(Default, Copy, Clone, Debug)]
pub struct UrlInfo {
	// scheme://domain:port(/path)[?(query_string)][(#fragment)]
	pub url_whole: Rdx,
	pub path: Rdx,
	pub query_string: Option<Rdx>,
	pub fragment: Option<Rdx>,
}

impl UrlInfo {
	pub fn parse_bytes(bytes: &[u8]) -> Result<UrlInfo, ()> {
		if bytes.is_empty() || bytes[0] != b'/' {
			return Err(());
		}

		let mut url_info = UrlInfo::default();

		let mut i = 1;

		while i < bytes.len() {
			if bytes[i] == b'?' || bytes[i] == b'#' {
				break;
			} else if !bytes[i].is_ascii_graphic() {
				return Err(());
			}
			i += 1;
		}

		url_info.path = Rdx::new(0, i);

		if i == bytes.len() {
			return Ok(url_info);
		}

		if bytes[i] == b'?' {
			i += 1;
			let query_string_start = i;
			while i < bytes.len() {
				if bytes[i] == b'#' {
					break;
				} else if !bytes[i].is_ascii_graphic() {
					return Err(())
				}
				i += 1;
			}

			if i != query_string_start {
				url_info.query_string = Some(Rdx::new(query_string_start, i));
			}

			if i == bytes.len() {
				return Ok(url_info);
			}
		}

		let fragment_start = i;
		while i < bytes.len() {
			if !bytes[i].is_ascii_graphic() {
				return Err(());
			}
			i += 1;
		}
		if i != fragment_start {
			url_info.fragment = Some(Rdx::new(fragment_start, i));
		}

		Ok(url_info)
	}
}

pub mod url_codec {
	pub fn url_decode(bytes: &[u8], dest_buffer: &mut [u8])
					  -> Result<usize, usize> {
		let mut buffer_too_small = false;
		let mut buf_head = 0;
		let mut i = 0;
		while i < bytes.len() {
			if !buffer_too_small && buf_head == dest_buffer.len() {
				buffer_too_small = true;
			}

			if bytes[i] == b'+' {
				if !buffer_too_small {
					dest_buffer[buf_head] = b' ';
				}
				buf_head += 1;
				i += 1;
			} else if bytes[i] == b'%' {
				if i + 2 >= bytes.len() {
					break;
				}
				match hex(bytes[i + 1], bytes[i + 2]) {
					None => {}
					Some(v) => {
						if !buffer_too_small {
							dest_buffer[buf_head] = v;
						}
						buf_head += 1;
						i += 3;
					}
				}
			} else {
				if !buffer_too_small {
					dest_buffer[buf_head] = bytes[i];
				}
				buf_head += 1;
				i += 1;
			}
		}

		if buffer_too_small {
			Err(buf_head)
		} else {
			Ok(buf_head)
		}
	}

	pub fn url_decode_to_vec(bytes: &[u8]) -> Vec<u8> {
		let mut s = Vec::new();
		let mut i = 0;
		while i < bytes.len() {
			if bytes[i] == b'%' {
				if i + 2 >= bytes.len() {
					break;
				}
				match hex(bytes[i + 1], bytes[i + 2]) {
					None => {}
					Some(v) => {
						s.push(v);
						i += 3;
					}
				}
			} else {
				s.push(bytes[i]);
				i += 1;
			}
		}
		s
	}

	pub fn url_encode(bytes: &[u8], dest_buffer: &mut [u8]) -> Result<usize, usize> {
		let mut buffer_too_small = false;
		let mut buf_head = 0usize;
		let mut count = 0usize;
		let mut i = 0;
		while i < bytes.len() {
			// match bytes[i] {
			//
			// }
		}
		todo!()
	}

	pub fn url_encode_to_vec(bytes: &[u8]) -> Vec<u8> {
		todo!()
	}

	fn hex(b1: u8, b2: u8) -> Option<u8> {
		let hi = match b1 {
			b'a'..b'f' => b1 - b'a' + 10,
			b'A'..b'F' => b1 - b'A' + 10,
			b'0'..b'9' => b1 - b'0',
			_ => return None
		};
		let lo = match b2 {
			b'a'..b'f' => b2 - b'a' + 10,
			b'A'..b'F' => b2 - b'A' + 10,
			b'0'..b'9' => b2 - b'0',
			_ => return None
		};
		Some(hi << 4 | lo)
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_hex() {
			assert_eq!(hex(b'2', b'0'), Some(0x20));
			assert_eq!(hex(b'a', b'a'), Some(0xaa));
			assert_eq!(hex(b'C', b'C'), Some(0xcc));
			assert_eq!(hex(b'z', b'1'), None);
		}

		#[test]
		fn decode() {
			let mut buf = [0u8; 20];
			assert_eq!(url_decode(b"hello%20world", &mut buf), Ok(11));
			assert_eq!(&buf[0..11], b"hello world");
			assert_eq!(url_decode(b"123456789%20123456789%20XYZ", &mut buf), Err(23));
			assert_eq!(&buf[0..20], b"123456789 123456789 ");
			assert_eq!(url_decode(b"%20%20%20", &mut buf), Ok(3));
			assert_eq!(&buf[0..3], b"   ");
			assert_eq!(url_decode(b"a+b+c", &mut buf), Ok(5));
			assert_eq!(&buf[0..5], b"a b c");

			assert_eq!(url_decode_to_vec(b"hello%20world"), Vec::from(b"hello world"));
			assert_eq!(url_decode_to_vec(b"123456789%20123456789%20XYZ"),
					   Vec::from(b"123456789 123456789 XYZ"));
			assert_eq!(url_decode_to_vec(b"%20%20%20"), Vec::from(b"   "));
		}
	}
}
