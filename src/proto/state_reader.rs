use crate::proto::rdx::Rdx;

pub struct StateReader {
	pub base: usize,
	pub head: usize,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Poll<T> {
	Ready(T),
	Pending,
}

impl StateReader {
	pub fn new() -> Self {
		Self {
			base: 0,
			head: 0,
		}
	}
	pub fn take_line(&mut self, buffer: &[u8]) -> Poll<Rdx> {
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
				return Poll::Ready(Rdx::new(from, to));
			}
			self.head += 1;
		}

		Poll::Pending
	}

	pub fn take_attachment(&mut self, buffer: &[u8], boundary: &[u8])
						   -> Poll<BoundaryInfo> {
		while self.head < buffer.len() {
			let mut idx = self.head;
			self.head += 1;
			let mut is_last = false;
			if buffer[idx] != b'\n' {
				continue;
			}
			if idx == 0 {
				continue
			}
			idx -= 1;
			if buffer[idx] == b'\r' {
				if idx == 0 {
					continue
				}
				idx -= 1;
			}
			if idx > 1 && buffer[idx] == b'-' && buffer[idx - 1] == b'-' {
				is_last = true;
				idx -= 2;
			}
			if idx + 1 < boundary.len() + 2 {
				continue;
			}
			idx = idx + 1 - boundary.len() - 2;

			if &buffer[idx + 2..idx + 2 + boundary.len()] != boundary {
				continue;
			}
			if &buffer[idx..idx + 2] != b"--" {
				continue;
			}

			if idx == 0 || buffer[idx - 1] != b'\n' {
				continue;
			}
			idx -= 1;
			if idx > 0 && buffer[idx - 1] == b'\r' {
				idx -= 1;
			}

			let tmp_base = self.base;
			self.base = self.head;
			return Poll::Ready(BoundaryInfo {
				data: Rdx::new(tmp_base, idx),
				is_last,
			});
		}
		Poll::Pending
	}
}

pub struct BoundaryInfo {
	pub data: Rdx,
	pub is_last: bool,
}


#[test]
fn simple_take_line() {
	let sample = b"line 1\r\nline 2\nline 3\r\nincomplete";
	let mut r = StateReader::new();
	assert_eq!(r.take_line(sample), Poll::Ready(Rdx::new(0, 6)));
	assert_eq!(r.take_line(sample), Poll::Ready(Rdx::new(8, 14)));
	assert_eq!(r.take_line(sample), Poll::Ready(Rdx::new(15, 21)));
	assert_eq!(r.take_line(sample), Poll::Pending);
}


#[test]
fn take_attachment() {
	let sample = b"somedata\r\n--abc\r\nanotherdata\r\n--abc--\r\n";
	let mut r = StateReader::new();
	match r.take_attachment(sample, b"abc") {
		Poll::Pending => panic!(),
		Poll::Ready(bi) => {
			assert_eq!(bi.data.get(sample), b"somedata");
			assert_eq!(bi.is_last, false);
		}
	};
	match r.take_attachment(sample, b"abc") {
		Poll::Pending => panic!(),
		Poll::Ready(bi) => {
			assert_eq!(bi.data.get(sample), b"anotherdata");
			assert_eq!(bi.is_last, true);
		}
	};
}
