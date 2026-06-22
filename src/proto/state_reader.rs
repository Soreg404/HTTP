use index_slice::IndexSlice;

#[derive(Default)]
pub struct StateReader {
	pub base: usize,
	pub head: usize,
}

impl StateReader {
	pub fn take_line(&mut self, buffer: &[u8]) -> Option<IndexSlice> {
		while self.head < buffer.len() {
			if buffer[self.head] == b'\n' {
				let to;
				if self.head > 0 && buffer[self.head - 1] == b'\r' {
					to = self.head - 1;
				} else {
					to = self.head;
				}
				let from = self.base;

				self.head += 1;
				self.base = self.head;
				return Some(IndexSlice::new(from, to));
			}
			self.head += 1;
		}

        None
	}

    pub fn take_exact(&mut self, buffer: &[u8], length: usize) -> Option<IndexSlice> {
        if self.base + length > buffer.len() {
            self.head = buffer.len();
            None
        } else {
            self.head += length;
            let tmp_base = self.base;
            self.base = self.head;
            Some(IndexSlice::new(tmp_base, self.head))
        }
    }

    pub fn take_attachment(&mut self, buffer: &[u8], boundary: &[u8])
        -> Option<BoundaryInfo> {
            let mut is_last = false;
            while self.head < buffer.len() {
                self.head += 1;
                let s = &buffer[self.base..self.head];

                let s = match s.strip_suffix(b"\n") {
                    None => continue,
                    Some(v) => v
                };
                let s = s.strip_suffix(b"\r").unwrap_or(s);
                let s = match s.strip_suffix(b"--") {
                    None => s,
                    Some(v) => {
                        is_last = true;
                        v
                    }
                };
                let s = match s.strip_suffix(boundary) {
                    None => continue,
                    Some(v) => v
                };
                let s = match s.strip_suffix(b"--") {
                    None => continue,
                    Some(v) => v
                };
                let s = match s.strip_suffix(b"\n") {
                    None => s,
                    Some(s) => s.strip_suffix(b"\r").unwrap_or(s)
                };

                let tmp_base = self.base;
                self.base = self.head;
                return Some(BoundaryInfo {
                    content: IndexSlice::new(tmp_base, tmp_base + s.len()),
                    is_last,
                });

            }
            None
	}
}

pub struct BoundaryInfo {
	pub content: IndexSlice,
	pub is_last: bool,
}


#[test]
fn simple_take_line() {
	let sample = b"line 1\r\nline 2\nline 3\r\nincomplete";
	let mut r = StateReader::default();
	assert_eq!(r.take_line(sample), Some(IndexSlice::new(0, 6)));
	assert_eq!(r.take_line(sample), Some(IndexSlice::new(8, 14)));
	assert_eq!(r.take_line(sample), Some(IndexSlice::new(15, 21)));
	assert_eq!(r.take_line(sample), None);
}


#[test]
fn take_attachment() {
	let sample = b"somedata\r\n--abc\r\nanotherdata\r\n--abc--\r\n";
	let mut r = StateReader::default();
	match r.take_attachment(sample, b"abc") {
		None => panic!(),
		Some(bi) => {
			assert_eq!(bi.content.as_slice_of(sample), b"somedata");
			assert_eq!(bi.is_last, false);
		}
	};
    assert_eq!(r.base, 17);
    assert_eq!(r.head, r.base);
	match r.take_attachment(sample, b"abc") {
		None => panic!(),
		Some(bi) => {
			assert_eq!(bi.content.as_slice_of(sample), b"anotherdata");
			assert_eq!(bi.is_last, true);
		}
	};
}
