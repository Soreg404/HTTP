use DelayedConsumeResult::*;

pub struct DelayedStateBuffer {
	current_read_head: usize,
	n_bytes_consumed: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum DelayedConsumeResult<'a> {
	NotEnoughBytes,
	Finished {
		base_index: usize,
		consumed: usize,
		slice: &'a [u8],
	},
}

impl DelayedStateBuffer {
	pub fn new() -> Self {
		Self {
			current_read_head: 0,
			n_bytes_consumed: 0,
		}
	}

	pub fn seek_byte(&mut self, buffer: &[u8]) -> Option<u8> {
		buffer.get(self.current_read_head).copied()
	}

	pub fn take_byte(&mut self, buffer: &[u8]) -> Option<u8> {
		self.seek_byte(buffer)
			.map(|b| {
				self.current_read_head += 1;
				b
			})
	}

	pub fn take_whitespace<'a>(&mut self, buffer: &'a [u8]) -> DelayedConsumeResult<'a> {
		let mut wh_len = 0usize;
		loop {
			match self.seek_byte(buffer) {
				Some(b) => {
					if !b.is_ascii() {
						panic!("not ascii character in sequence");
					}
					if b.is_ascii_whitespace() {
						wh_len += 1;
						self.take_byte(buffer);
					} else {
						let tmp_consumed = self.n_bytes_consumed;
						self.n_bytes_consumed = self.current_read_head;
						return Finished {
							base_index: tmp_consumed,
							consumed: wh_len,
							slice: &buffer[tmp_consumed..wh_len],
						};
					}
				}
				None => return NotEnoughBytes
			}
		}
	}

	pub fn take_line<'a>(&mut self, buffer: &'a [u8]) -> DelayedConsumeResult<'a> {
		while let Some(b) = self.take_byte(buffer) {
			if !b.is_ascii() {
				panic!("Non-ascii character in sequence");
			}

			if b == b'\n' {
				let slice = &buffer[self.n_bytes_consumed..self.current_read_head - 1];
				let slice = slice.strip_suffix(b"\r").unwrap_or(slice);

				let tmp_consumed = self.n_bytes_consumed;
				self.n_bytes_consumed = self.current_read_head;
				return Finished {
					base_index: tmp_consumed,
					consumed: self.current_read_head - tmp_consumed,
					slice,
				};
			}
		}

		NotEnoughBytes
	}

	pub fn take_exact<'a>(&mut self, buffer: &'a [u8], length: usize) -> DelayedConsumeResult<'a> {
		if self.n_bytes_consumed + length <= buffer.len() {
			let tmp_consumed = self.n_bytes_consumed;
			self.n_bytes_consumed = self.n_bytes_consumed + length;
			self.current_read_head = self.n_bytes_consumed;
			Finished {
				base_index: tmp_consumed,
				consumed: length,
				slice: &buffer[tmp_consumed..self.n_bytes_consumed],
			}
		} else {
			NotEnoughBytes
		}
	}
}

impl DelayedStateBuffer {
	pub fn consumed(&self) -> usize {
		self.n_bytes_consumed
	}
	pub fn current_read_head(&self) -> usize {
		self.current_read_head
	}
}

#[test]
fn test_buffer_read() {
	let mut internal_buffer = Vec::<u8>::from(
		b"HTTP/1.1 200 OK\r\n\
			host: unstd.pl\r\n\r\n"
	);
	{
		let mut buffer_reader_line = DelayedStateBuffer::new();
		match buffer_reader_line.take_line(&internal_buffer) {
			NotEnoughBytes => panic!(),
			Finished { base_index, consumed, slice } => {
				assert_eq!(base_index, 0);
				assert_eq!(consumed, 17);
				assert_eq!(slice, b"HTTP/1.1 200 OK");
			}
		}
	}

	{
		let mut buffer_reader_exact = DelayedStateBuffer::new();
		match buffer_reader_exact.take_exact(&internal_buffer, 4) {
			NotEnoughBytes => panic!(),
			Finished { base_index, consumed, slice } => {
				assert_eq!(base_index, 0);
				assert_eq!(consumed, 4);
				assert_eq!(slice, b"HTTP");
			}
		}

		let mut buffer_reader_next_exact = buffer_reader_exact;
		match buffer_reader_next_exact.take_exact(&internal_buffer, 4) {
			NotEnoughBytes => panic!(),
			Finished { base_index, consumed, slice } => {
				assert_eq!(base_index, 4);
				assert_eq!(consumed, 4);
				assert_eq!(slice, b"/1.1");
			}
		}
	}

	{
		let mut buffer_reader_not_enough = DelayedStateBuffer::new();
		buffer_reader_not_enough.take_line(&internal_buffer);
		buffer_reader_not_enough.take_line(&internal_buffer);
		match buffer_reader_not_enough.take_line(&internal_buffer) {
			NotEnoughBytes => panic!(),
			Finished { base_index, consumed, slice } => {
				assert_eq!(base_index, 33);
				assert_eq!(consumed, 2);
				assert_eq!(slice, b"");
			}
		}

		assert_eq!(buffer_reader_not_enough.take_line(&internal_buffer), NotEnoughBytes);
	}
}
