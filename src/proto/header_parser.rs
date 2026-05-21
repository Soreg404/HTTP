use crate::proto::rdx::Rdx;

pub struct HeaderParser<'a> {
	target_line: &'a [u8],
	name_end: usize,
	body_start: usize,
	body_end: usize,
	head: usize,
}

impl<'a> HeaderParser<'a> {
	pub fn new(target_line: &'a [u8]) -> Result<Self, ()> {
		let line = target_line.strip_suffix(b"\n")
			.unwrap_or(target_line);
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

		Ok(Self {
			target_line,
			name_end,
			body_start,
			body_end,
			head: body_start,
		})
	}
}

impl HeaderParser<'_> {
	pub fn field_name(&self) -> Rdx {
		Rdx::new(0, self.name_end)
	}

	pub fn field_body(&self) -> Rdx {
		Rdx::new(self.body_start, self.body_end)
	}

	fn skip_ws(&mut self) {
		while self.head < self.target_line.len() {
			if self.target_line[self.head] != b' ' && self.target_line[self.head] != b'\t' {
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

	pub fn next_atom(&mut self) -> Option<Rdx> {
		let word_start = self.head;
		let mut word_end = self.target_line.len();
		while self.head < self.target_line.len() {
			let c = self.target_line[self.head];
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

	fn next_word(&mut self) -> Result<Option<Rdx>, ()> {
		if self.target_line[self.head] == b'"' {
			self.head += 1;
			let quot_start = self.head;
			while self.head < self.target_line.len() {
				if self.target_line[self.head] == b'"' {
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

	pub fn next_type(&mut self) -> Result<RdxType, ()> {
		match self.next_atom() {
			None => Err(()),
			Some(w) => {
				let ws = w.get(self.target_line);
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
				Ok(RdxType {
					main: Rdx::new(0, slash_pos).with_base(w.from()),
					sub: Rdx::new(slash_pos + 1, w.len()).with_base(w.from()),
				})
			}
		}
	}

	pub fn next_attribute(&mut self) -> Option<Result<RdxAttribute, ()>> {
		if self.head == self.target_line.len() ||
			self.target_line[self.head] != b';' {
			return None;
		}
		self.head += 1;
		self.skip_ws();

		let key_start = self.head;
		let mut key_end = None::<usize>;
		let mut eq_sign_pos = None::<usize>;
		while self.head < self.target_line.len() {
			let c = self.target_line[self.head];
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

		if self.target_line[self.head] != b'=' {
			return Some(Err(()));
		}
		self.head += 1;
		self.skip_ws();

		let value = match self.next_word() {
			Err(()) => return Some(Err(())),
			Ok(None) => return Some(Err(())),
			Ok(Some(w)) => w
		};

		Some(Ok(RdxAttribute {
			key,
			value,
		}))
	}
}

pub struct RdxType {
	pub main: Rdx,
	pub sub: Rdx,
}

pub struct RdxAttribute {
	pub key: Rdx,
	pub value: Rdx,
}

#[test]
fn header_parser_new() {
	let line = b"host:     localhost    ";
	match HeaderParser::new(line) {
		Err(()) => panic!(),
		Ok(hp) => {
			assert_eq!(hp.name_end, 4);
			assert_eq!(hp.body_start, 10);
			assert_eq!(hp.body_end, 19);

			assert_eq!(hp.field_name().get(line), b"host");
			assert_eq!(hp.field_body().get(line), b"localhost");
		}
	};
}

#[test]
fn header_parser_atoms() {
	let line = b"content-type:   application/rust     text/plain  random/bullshit   ";
	match HeaderParser::new(line) {
		Err(()) => panic!(),
		Ok(mut hp) => {
			assert_eq!(hp.field_name().get(line), b"content-type");
			assert_eq!(hp.field_body().get(line),
					   b"application/rust     text/plain  random/bullshit");

			let ct = hp.next_type().unwrap();
			assert_eq!(ct.main.get(line), b"application");
			assert_eq!(ct.sub.get(line), b"rust");

			let ct = hp.next_type().unwrap();
			assert_eq!(ct.main.get(line), b"text");
			assert_eq!(ct.sub.get(line), b"plain");

			let ct = hp.next_type().unwrap();
			assert_eq!(ct.main.get(line), b"random");
			assert_eq!(ct.sub.get(line), b"bullshit");
		}
	};
}

#[test]
fn next_word() {
	let line = b"header: atom \"quoted text\"";
	match HeaderParser::new(line) {
		Err(()) => panic!(),
		Ok(mut hp) => {
			assert_eq!(hp.next_word().unwrap().unwrap().get(line), b"atom");
			assert_eq!(hp.next_word().unwrap().unwrap().get(line), b"quoted text");
		}
	}
}

#[test]
fn next_attribute() {
	let line = b"header: start; first=attrib  ;  second = \"characteristic\"";
	match HeaderParser::new(line) {
		Err(()) => panic!(),
		Ok(mut hp) => {
			let word = hp.next_word().unwrap().unwrap();
			assert_eq!(word.get(line), b"start");

			let a = hp.next_attribute().unwrap().unwrap();
			assert_eq!(a.key.get(line), b"first");
			assert_eq!(a.value.get(line), b"attrib");

			let a = hp.next_attribute().unwrap().unwrap();
			assert_eq!(a.key.get(line), b"second");
			assert_eq!(a.value.get(line), b"characteristic");
		}
	}
}
