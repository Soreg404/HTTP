use std::ops::Deref;
use crate::proto::rdx::Rdx;

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
					}
				}
				i += 3;
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

			assert_eq!(url_decode_to_vec(b"hello%20world"), Vec::from(b"hello world"));
			assert_eq!(url_decode_to_vec(b"123456789%20123456789%20XYZ"),
					   Vec::from(b"123456789 123456789 XYZ"));
			assert_eq!(url_decode_to_vec(b"%20%20%20"), Vec::from(b"   "));
			assert_eq!(url_decode_to_vec(b"cze%C5%9B%C4%87").as_slice(), "cześć".as_bytes());
			assert_eq!(url_decode_to_vec(b"%22").as_slice(), b"\"");
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
		pub fn decoded_to_vec(self)
							  -> UrlPartsIteratorDecodeVec<'a> {
			UrlPartsIteratorDecodeVec {
				it: self,
			}
		}
		pub fn decoded_to_buffer<'b>(self, buffer: &'b mut [u8])
									 -> UrlPartsDecodeBuf<'a, 'b> {
			UrlPartsDecodeBuf {
				it: self,
				buffer,
			}
		}
	}
	impl<'a> Iterator for UrlPartsIterator<'a> {
		type Item = &'a [u8];

		fn next(&mut self) -> Option<Self::Item> {
			if self.head == self.target.len() {
				return None;
			}

			while self.head < self.target.len() && self.target[self.head] == b'/' {
				self.head += 1;
			}
			if self.head == self.target.len() {
				return None;
			}

			let part_start = self.head;

			while self.head < self.target.len() && self.target[self.head] != b'/' {
				self.head += 1;
			}

			let part_end = self.head;

			if self.head < self.target.len() {
				self.head += 1;
			}

			Some(&self.target[part_start..part_end])
		}
	}

	pub struct UrlPartsIteratorDecodeVec<'a> {
		it: UrlPartsIterator<'a>,
	}
	impl Iterator for UrlPartsIteratorDecodeVec<'_> {
		type Item = Vec<u8>;
		fn next(&mut self) -> Option<Self::Item> {
			Some(super::url_codec::url_decode_to_vec(self.it.next()?))
		}
	}

	pub struct UrlPartsDecodeBuf<'a, 'b> {
		it: UrlPartsIterator<'a>,
		buffer: &'b mut [u8],
	}
	impl UrlPartsDecodeBuf<'_, '_> {
		pub fn next(&mut self) -> Option<Result<&[u8], usize>> {
			Some(super::url_codec::url_decode(self.it.next()?, &mut self.buffer))
		}
	}

	#[cfg(test)]
	mod url_parts_it_tests {
		use super::UrlPartsIterator;

		#[test]
		fn t1() {
			let target = b"/cze%C5%9B%C4%87////cz%C4%99%C5%9B%C4%87///";
			let mut it = UrlPartsIterator::new(target);
			assert_eq!(it.next(), Some(b"cze%C5%9B%C4%87".as_slice()));
			assert_eq!(it.next(), Some(b"cz%C4%99%C5%9B%C4%87".as_slice()));

			let mut it = UrlPartsIterator::new(target)
				.decoded_to_vec();
			assert_eq!(it.next(), Some("cześć".as_bytes().to_vec()));
			assert_eq!(it.next(), Some("część".as_bytes().to_vec()));

			let mut buffer = [0u8; 20];
			let mut it = UrlPartsIterator::new(target)
				.decoded_to_buffer(&mut buffer);
			assert_eq!(it.next(), Some(Ok("cześć".as_bytes())));
			assert_eq!(it.next(), Some(Ok("część".as_bytes())));
		}
	}
}
