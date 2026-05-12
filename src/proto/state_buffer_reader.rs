use StateBufferReaderResult::*;

#[derive(Debug)]
pub struct BufferReader {
	current_read_head: usize,
	n_bytes_consumed: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum StateBufferReaderResult<T> {
	NotEnoughBytes,
	Done(T),
}

impl BufferReader {
	pub fn new() -> Self {
		Self {
			current_read_head: 0,
			n_bytes_consumed: 0,
		}
	}

	pub fn get_read_head(&self) -> usize {
		self.current_read_head
	}

	pub fn get_n_bytes_consumed(&self) -> usize {
		self.n_bytes_consumed
	}

	pub fn take_whitespace<'a>(&mut self, buffer: &'a [u8])
							   -> StateBufferReaderResult<&'a [u8]> {
		let mut wh_len = 0usize;
		loop {
			match buffer.get(self.current_read_head) {
				Some(b) => {
					if !b.is_ascii() {
						panic!("not ascii character in sequence");
					}
					if b.is_ascii_whitespace() {
						wh_len += 1;
						self.current_read_head += 1;
					} else {
						let tmp_consumed = self.n_bytes_consumed;
						self.n_bytes_consumed = self.current_read_head;
						return Done(&buffer[tmp_consumed..wh_len]);
					}
				}
				None => return NotEnoughBytes
			}
		}
	}

	pub fn take_line<'a>(&mut self, buffer: &'a [u8])
						 -> StateBufferReaderResult<&'a [u8]> {
		while let Some(b) = buffer.get(self.current_read_head) {
			self.current_read_head += 1;

			if !b.is_ascii() {
				panic!("Non-ascii character in sequence");
			}

			if *b == b'\n' {
				let line_strip_lf =
					&buffer[self.n_bytes_consumed..self.current_read_head - 1];
				let line_strip_crlf =
					line_strip_lf.strip_suffix(b"\r").unwrap_or(line_strip_lf);

				self.n_bytes_consumed = self.current_read_head;
				return Done(line_strip_crlf);
			}
		}

		NotEnoughBytes
	}

	pub fn take_exact<'a>(&mut self, buffer: &'a [u8], length: usize) -> StateBufferReaderResult<&'a [u8]> {
		if self.n_bytes_consumed + length <= buffer.len() {
			self.current_read_head += length;
			let take_slice =
				&buffer[self.n_bytes_consumed..self.current_read_head];
			self.n_bytes_consumed = self.current_read_head;
			Done(take_slice)
		} else {
			NotEnoughBytes
		}
	}

	pub fn reset_rest(&mut self, buffer: &mut [u8]) {
		let start_index = self.n_bytes_consumed;
		self.n_bytes_consumed = 0;
		self.current_read_head -= start_index;
		let mut counter = 0;
		while let Some(b) = buffer.get(start_index + counter) {
			let b = *b;

			buffer[counter] = b;

			counter += 1;
		}
	}

}

impl BufferReader {
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
		let mut buffer_reader_line = BufferReader::new();
		match buffer_reader_line.take_line(&internal_buffer) {
			NotEnoughBytes => panic!(),
			Done(line) => assert_eq!(line, b"HTTP/1.1 200 OK")
		}
	}

	{
		let mut buffer_reader_exact = BufferReader::new();
		match buffer_reader_exact.take_exact(&internal_buffer, 4) {
			NotEnoughBytes => panic!(),
			Done(line) => assert_eq!(line, b"HTTP")
		}

		let mut buffer_reader_next_exact = buffer_reader_exact;
		match buffer_reader_next_exact.take_exact(&internal_buffer, 4) {
			NotEnoughBytes => panic!(),
			Done(line) => assert_eq!(line, b"/1.1")
		}
	}

	{
		let mut buffer_reader_not_enough = BufferReader::new();
		buffer_reader_not_enough.take_line(&internal_buffer);
		buffer_reader_not_enough.take_line(&internal_buffer);
		match buffer_reader_not_enough.take_line(&internal_buffer) {
			NotEnoughBytes => panic!(),
			Done(line) => assert_eq!(line, b"")
		}

		assert_eq!(buffer_reader_not_enough.take_line(&internal_buffer), NotEnoughBytes);
	}
}
