use crate::request_collector::Rdx;

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

	fn is_ctl(b: u8) -> bool {
		match b {
			b'(' | b')' | b'<' | b'>' | b'@' | b',' |
			b';' | b':' | b'"' | b'.' | b'[' | b']' | b'\\' => true,
			_ => false,
		}
	}

	fn next_word(&mut self) -> Option<Rdx> {
		let word_start = self.head;
		let mut word_end = self.target_line.len();
		while self.head < self.target_line.len() {
			let c = self.target_line[self.head];
			if c <= 0x20 || c >= 0x7f || Self::is_ctl(c) {
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

	fn next_type(&mut self) -> Result<RdxType, ()> {
		match self.next_word() {
			None => Err(()),
			Some(w) => {
				let ws = w.get(self.target_line);
				let mut i = 0;
				let mut part = true;
				let mut slash_pos = 0;
				while i < w.len() {
					if ws[i] == b'/' {
						if part == false {
							return Err(())
						}
						part = false;
						slash_pos = i;
					}
					i += 1;
				}
				if slash_pos + 1 == w.len() {
					return Err(())
				}
				Ok(RdxType {
					main: Rdx::new(0, slash_pos).with_base(w.from()),
					sub: Rdx::new(slash_pos + 1, w.len()).with_base(w.from()),
				})
			}
		}
	}
}

pub struct RdxType {
	main: Rdx,
	sub: Rdx
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
fn header_parser_words() {
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
