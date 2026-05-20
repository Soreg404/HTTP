

#[derive(Default, Copy, Clone, Debug)]
pub struct UrlMeta {
	// scheme://domain:port[/path][?query_string][#fragment]
	//                      ^0     ^query_idx     ^frag_idx
	pub query_string: Option<usize>,
	pub fragment: Option<usize>,
}

fn is_allowed_byte(c: u8) -> bool {
	if c.is_ascii_alphanumeric() {
		return true;
	}
	match c {
		b';' | b'/' | b'?' | b':' | b'%' | b'@' | b'=' | b'&' | b'$' |
		b'-' | b'_' | b'.' | b'+' | b'!' | b'*' | b'(' | b')' | b',' |
		b'^' | b'{' | b'}' | b'|' | b'~' | b'[' | b']' | b'`' | b'\\' | b'\''
		=> true,
		_ => false
	}
}

impl UrlMeta {
	pub fn parse_bytes(bytes: &[u8]) -> Result<UrlMeta, ()> {
		if bytes.is_empty() || bytes[0] != b'/' {
			return Err(());
		}

		enum Part {
			Path,
			Query,
			Frag,
		}
		let mut part = Part::Path;

		let mut query_string = None::<usize>;
		let mut fragment = None::<usize>;

		for (i, b) in bytes.iter().enumerate() {
			if !is_allowed_byte(*b) {
				return Err(());
			}
			match part {
				Part::Path => {
					if *b == b'?' {
						part = Part::Query;
						query_string = Some(i);
					} else if *b == b'#' {
						part = Part::Frag;
						fragment = Some(i);
					}
				}
				Part::Query => {
					if *b == b'#' {
						part = Part::Frag;
						fragment = Some(i);
					}
				}
				Part::Frag => {}
			}
		}

		Ok(UrlMeta {
			query_string,
			fragment,
		})
	}
}

pub fn url_from_parts(meta: UrlMeta, buffer: &[u8]) -> Url {
	Url {
		meta,
		buffer,
	}
}

pub struct Url<'a> {
	meta: UrlMeta,
	buffer: &'a [u8],
}

impl Url<'_> {
	pub fn path(&self) -> &[u8] {
		let end = self.meta.query_string.unwrap_or(
			self.meta.fragment.unwrap_or(
				self.buffer.len()
			)
		);
		&self.buffer[0..end]
	}

	pub fn query(&self) -> Option<&[u8]> {
		match self.meta.query_string {
			None => None,
			Some(qr) => {
				let end = self.meta.fragment.unwrap_or(
					self.buffer.len()
				);
				Some(&self.buffer[qr..end])
			}
		}
	}

	pub fn fragment(&self) -> Option<&[u8]> {
		match self.meta.fragment {
			None => None,
			Some(fr) => {
				Some(&self.buffer[fr..])
			}
		}
	}

	pub fn decode(bytes: &[u8], dest_buffer: &mut [u8]) -> Result<usize, usize> {
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

	pub fn decode_to_vec(bytes: &[u8]) -> Vec<u8> {
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

	pub fn encode(bytes: &[u8], dest_buffer: &mut [u8]) -> Result<usize, usize> {
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

	pub fn encode_to_vec(bytes: &[u8]) -> Vec<u8> {
		todo!()
	}
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

#[test]
fn test_hex() {
	assert_eq!(hex(b'2', b'0'), Some(0x20));
	assert_eq!(hex(b'a', b'a'), Some(0xaa));
	assert_eq!(hex(b'C', b'C'), Some(0xcc));
	assert_eq!(hex(b'z', b'1'), None);
}

#[test]
fn url_decode() {
	let mut buf = [0u8; 20];
	assert_eq!(Url::decode(b"hello%20world", &mut buf), Ok(11));
	assert_eq!(&buf[0..11], b"hello world");
	assert_eq!(Url::decode(b"123456789%20123456789%20XYZ", &mut buf), Err(23));
	assert_eq!(&buf[0..20], b"123456789 123456789 ");
	assert_eq!(Url::decode(b"%20%20%20", &mut buf), Ok(3));
	assert_eq!(&buf[0..3], b"   ");
	assert_eq!(Url::decode(b"a+b+c", &mut buf), Ok(5));
	assert_eq!(&buf[0..5], b"a b c");

	assert_eq!(Url::decode_to_vec(b"hello%20world"), Vec::from(b"hello world"));
	assert_eq!(Url::decode_to_vec(b"123456789%20123456789%20XYZ"),
			   Vec::from(b"123456789 123456789 XYZ"));
	assert_eq!(Url::decode_to_vec(b"%20%20%20"), Vec::from(b"   "));
}
