use crate::proto::rdx::Rdx;
// use std::ops::Deref;

#[derive(Default, Copy, Clone, Debug)]
pub struct UrlInfo {
	// scheme://domain:port(/path)[?(query_string)][(#fragment)]
	pub url_whole: Rdx,
	pub path: Rdx,
	pub query_string: Option<Rdx>,

	// todo: Fragment in url might be an error actually...
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
		url_info.fragment = Some(Rdx::new(fragment_start, i));

		Ok(url_info)
	}
}

pub mod url_codec {
	pub fn url_decode<'a, 'b>(
		bytes: &'a [u8],
		dest_buffer: &'b mut [u8],
	)
		-> Result<&'b [u8], usize> {
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
			Ok(&dest_buffer[..buf_head])
		}
	}

	pub fn url_decode_to_vec<'a, 'b>(bytes: &'a [u8], vec: &'b mut Vec<u8>) -> &'b [u8] {
		let mut i = 0;
		let start = vec.len();
		while i < bytes.len() {
			if bytes[i] == b'%' {
				if i + 2 >= bytes.len() {
					break;
				}
				match hex(bytes[i + 1], bytes[i + 2]) {
					None => {}
					Some(v) => {
						vec.push(v);
					}
				}
				i += 3;
			} else {
				vec.push(bytes[i]);
				i += 1;
			}
		}
		&vec[start..]
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
			b'a'..=b'f' => b1 - b'a' + 10,
			b'A'..=b'F' => b1 - b'A' + 10,
			b'0'..=b'9' => b1 - b'0',
			_ => return None
		};
		let lo = match b2 {
			b'a'..=b'f' => b2 - b'a' + 10,
			b'A'..=b'F' => b2 - b'A' + 10,
			b'0'..=b'9' => b2 - b'0',
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
			assert_eq!(url_decode(b"hello%20world", &mut buf), Ok(b"hello world".as_slice()));
			assert_eq!(url_decode(b"123456789%20123456789%20XYZ", &mut buf), Err(23));
			assert_eq!(&buf[0..20], b"123456789 123456789 ");
			assert_eq!(url_decode(b"%20%20%20", &mut buf), Ok(b"   ".as_slice()));
			assert_eq!(url_decode(b"a+b+c", &mut buf), Ok(b"a b c".as_slice()));

			let mut v = Vec::new();
			assert_eq!(url_decode_to_vec(b"hello%20world", &mut v), b"hello world");
			assert_eq!(url_decode_to_vec(b"123456789%20123456789%20XYZ", &mut v),
					   b"123456789 123456789 XYZ");
			assert_eq!(url_decode_to_vec(b"%20%20%20", &mut v), b"   ");
			assert_eq!(url_decode_to_vec(b"cze%C5%9B%C4%87", &mut v), "cześć".as_bytes());
			assert_eq!(url_decode_to_vec(b"%22", &mut v), b"\"");
		}
	}
}

pub mod url_iter {
	pub struct UrlPartsIterator<'a> {
		target: &'a [u8],
		head: usize,
	}
	impl<'a> UrlPartsIterator<'a> {
		pub fn new(encoded_url_path: &'a [u8]) -> Self {
			Self {
				target: encoded_url_path.strip_prefix(b"/")
					.unwrap_or(encoded_url_path),
				head: 0,
			}
		}
	}
	impl<'a> Iterator for UrlPartsIterator<'a> {
		type Item = UrlPart<'a>;

		fn next(&mut self) -> Option<Self::Item> {
			while self.head < self.target.len()
				&& self.target[self.head] == b'/' {
				self.head += 1;
			}
			if self.head == self.target.len() {
				return None;
			}

			let part_start = self.head;

			while self.head < self.target.len()
				&& self.target[self.head] != b'/' {
				self.head += 1;
			}

			Some(UrlPart {
				s: &self.target[part_start..self.head]
			})
		}
	}
	pub struct UrlPart<'a> {
		s: &'a [u8],
	}
	impl<'a> UrlPart<'a> {
		pub fn raw(&self) -> &[u8] {
			self.s
		}
		pub fn decode_to_buf<'b>(&self, buffer: &'b mut [u8])
								 -> Result<&'b [u8], usize> {
			super::url_codec::url_decode(self.s, buffer)
		}
		pub fn decode_to_vec<'b>(&self, vec: &'b mut Vec<u8>) -> &'b [u8] {
			super::url_codec::url_decode_to_vec(self.s, vec)
		}
	}

	#[test]
	fn url_parts_it_test() {
		let target = b"/cze%C5%9B%C4%87////cz%C4%99%C5%9B%C4%87///";
		let mut it = UrlPartsIterator::new(target);
		assert_eq!(it.next().unwrap().raw(), b"cze%C5%9B%C4%87");
		assert_eq!(it.next().unwrap().raw(), b"cz%C4%99%C5%9B%C4%87");

		let mut v = Vec::new();
		let mut it = UrlPartsIterator::new(target);
		assert_eq!(it.next().unwrap().decode_to_vec(&mut v), "cześć".as_bytes());
		assert_eq!(it.next().unwrap().decode_to_vec(&mut v), "część".as_bytes());

		let mut buffer = [0u8; 20];
		let mut it = UrlPartsIterator::new(target);
		assert_eq!(it.next().unwrap().decode_to_buf(&mut buffer), Ok("cześć".as_bytes()));
		assert_eq!(it.next().unwrap().decode_to_buf(&mut buffer), Ok("część".as_bytes()));
	}
}
