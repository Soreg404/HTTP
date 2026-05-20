use crate::request_collector::Rdx;

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
