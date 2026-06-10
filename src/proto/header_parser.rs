use crate::proto::rdx::Rdx;

pub struct HeaderRdx {
	pub name: Rdx,
	pub body: Rdx,
}


pub fn header_from_line(line: &[u8])
						-> Result<HeaderRdx, ()> {
	if line.is_empty() {
		return Err(());
	}
	let line = line.strip_suffix(b"\n")
		.unwrap_or(line);
	let line = line.strip_suffix(b"\r")
		.unwrap_or(line);


	let mut i = 0;
	let mut name_end = 0;
	while i < line.len() {
		if !line[i].is_ascii_graphic() {
			return Err(());
		} else if line[i] == b':' {
			name_end = i;
			i += 1;
			break;
		}
		i += 1;
	}

	while i < line.len() {
		if !line[i].is_ascii_whitespace() {
			break;
		}
		i += 1;
	}
	let body_start = i;

	let mut body_end = line.len();
	while body_end > body_start {
		if !line[body_end - 1].is_ascii_whitespace() {
			break;
		}
		body_end -= 1;
	}
	if body_start == body_end {
		return Err(());
	}

	Ok(HeaderRdx {
		name: Rdx::new(0, name_end),
		body: Rdx::new(body_start, body_end),
	})
}

pub struct HeaderBodyParser<'a> {
	target_bytes: &'a [u8],
	head: usize,
}

impl<'a> HeaderBodyParser<'a> {
	pub fn new(value_bytes: &'a [u8]) -> Self {
		Self {
			target_bytes: value_bytes,
			head: 0,
		}
	}

	pub fn next_atom(&mut self) -> Option<Rdx> {
		let word_start = self.head;
		let mut word_end = self.target_bytes.len();
		while self.head < self.target_bytes.len() {
			let c = self.target_bytes[self.head];
			if c <= 0x20 || c >= 0x7f || Self::is_special(c) {
				word_end = self.head;
				break;
			}
			self.head += 1;
		}

		self.skip_ws();

		if word_start == word_end {
			None
		} else {
			Some(Rdx::new(word_start, word_end))
		}
	}

	pub fn next_type(&mut self) -> Result<TypeRdx, ()> {
		match self.next_atom() {
			None => Err(()),
			Some(w) => {
				let ws = w.get(self.target_bytes);
				let mut i = 0;
				let mut part = true;
				let mut slash_pos = 0;
				while i < w.len() {
					if ws[i] == b'/' {
						if part == false {
							return Err(());
						}
						part = false;
						slash_pos = i;
					}
					i += 1;
				}
				if slash_pos + 1 == w.len() {
					return Err(());
				}
				Ok(TypeRdx {
					main: Rdx::new(0, slash_pos).offset(w.from()),
					sub: Rdx::new(slash_pos + 1, w.len()).offset(w.from()),
				})
			}
		}
	}

	pub fn next_attribute(&mut self) -> Option<Result<AttributeRdx, ()>> {
		if self.head == self.target_bytes.len() ||
			self.target_bytes[self.head] != b';' {
			return None;
		}
		self.head += 1;
		self.skip_ws();

		let key_start = self.head;
		let mut key_end = None::<usize>;
		let mut eq_sign_pos = None::<usize>;
		while self.head < self.target_bytes.len() {
			let c = self.target_bytes[self.head];
			if c == b' ' {
				if key_end.is_none() {
					key_end = Some(self.head);
				}
			} else if !c.is_ascii_graphic() {
				return Some(Err(()));
			} else if c == b'=' {
				if key_end.is_none() {
					key_end = Some(self.head);
				}
				eq_sign_pos = Some(self.head);
				break;
			}
			self.head += 1;
		}
		if eq_sign_pos.is_none() {
			return Some(Err(()));
		}

		let key = Rdx::new(key_start, key_end.unwrap());

		if self.target_bytes[self.head] != b'=' {
			return Some(Err(()));
		}
		self.head += 1;
		self.skip_ws();

		let value = match self.next_word() {
			Err(()) => return Some(Err(())),
			Ok(None) => return Some(Err(())),
			Ok(Some(w)) => w
		};

		Some(Ok(AttributeRdx {
			key,
			value,
		}))
	}

	fn next_word(&mut self) -> Result<Option<Rdx>, ()> {
		if self.target_bytes[self.head] == b'"' {
			self.head += 1;
			let quot_start = self.head;
			while self.head < self.target_bytes.len() {
				if self.target_bytes[self.head] == b'"' {
					let quot_end = self.head;
					self.head += 1;
					self.skip_ws();
					return Ok(Some(Rdx::new(quot_start, quot_end)));
				}
				self.head += 1;
			}
			Err(())
		} else {
			Ok(self.next_atom())
		}
	}

	fn skip_ws(&mut self) {
		while self.head < self.target_bytes.len() {
			if self.target_bytes[self.head] != b' ' && self.target_bytes[self.head] != b'\t' {
				return;
			}
			self.head += 1;
		}
	}

	fn is_special(b: u8) -> bool {
		match b {
			b'(' | b')' | b'<' | b'>' | b'@' | b',' |
			b';' | b':' | b'"' | b'.' | b'[' | b']' | b'\\' => true,
			_ => false,
		}
	}
}

pub struct TypeRdx {
	pub main: Rdx,
	pub sub: Rdx,
}

pub struct AttributeRdx {
	pub key: Rdx,
	pub value: Rdx,
}

#[test]
fn header_parser_new() {
	let line = b"host:     localhost    ";
	let h = header_from_line(line).unwrap();
	assert_eq!(h.name.get(line), b"host");
	assert_eq!(h.body.get(line), b"localhost");
	let field_body_bytes = h.body.get(line);
	let mut hp = HeaderBodyParser::new(field_body_bytes);
	match hp.next_atom() {
		None => panic!(),
		Some(rdx) => {
			assert_eq!(rdx.get(field_body_bytes), b"localhost");
		}
	}
}

#[test]
fn header_parser_atoms() {
	let line = b"content-type:   application/rust     text/plain  random/bullshit   ";
	let h = header_from_line(line).unwrap();
	assert_eq!(h.name.get(line), b"content-type");
	assert_eq!(h.body.get(line),
			   b"application/rust     text/plain  random/bullshit");
	let field_body_bytes = h.body.get(line);
	let mut hbp = HeaderBodyParser::new(field_body_bytes);
		let ct = hbp.next_type().unwrap();
		assert_eq!(ct.main.get(field_body_bytes), b"application");
		assert_eq!(ct.sub.get(field_body_bytes), b"rust");

		let ct = hbp.next_type().unwrap();
		assert_eq!(ct.main.get(field_body_bytes), b"text");
		assert_eq!(ct.sub.get(field_body_bytes), b"plain");

		let ct = hbp.next_type().unwrap();
		assert_eq!(ct.main.get(field_body_bytes), b"random");
		assert_eq!(ct.sub.get(field_body_bytes), b"bullshit");
}

#[test]
fn next_word() {
	let line = b"header: atom \"quoted text\"";
	let h = header_from_line(line).unwrap();
	let field_body_bytes = h.body.get(line);
	let mut hbp = HeaderBodyParser::new(field_body_bytes);
	assert_eq!(hbp.next_word().unwrap().unwrap().get(field_body_bytes), b"atom");
	assert_eq!(hbp.next_word().unwrap().unwrap().get(field_body_bytes), b"quoted text");
}

#[test]
fn t_next_attribute() {
	let line = b"header: start; first=attrib  ;  second = \"characteristic\"";
	let h = header_from_line(line).unwrap();
	let field_body_bytes = h.body.get(line);
	let mut hbp = HeaderBodyParser::new(field_body_bytes);

	let word = hbp.next_word().unwrap().unwrap();
	assert_eq!(word.get(field_body_bytes), b"start");

	let a = hbp.next_attribute().unwrap().unwrap();
	assert_eq!(a.key.get(field_body_bytes), b"first");
	assert_eq!(a.value.get(field_body_bytes), b"attrib");

	let a = hbp.next_attribute().unwrap().unwrap();
	assert_eq!(a.key.get(field_body_bytes), b"second");
	assert_eq!(a.value.get(field_body_bytes), b"characteristic");
}
