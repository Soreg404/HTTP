use std::io::Write;

#[derive(Default)]
struct ReaderState {
	interruptible_read_head: usize,
	bytes_taken: usize,
}

#[derive(Default)]
pub struct BufferReader {
	internal_buffer: Vec<u8>,
	state: ReaderState,
}

pub struct BufferReaderRef<'itl_buf> {
	internal_buffer: &'itl_buf Vec<u8>,
	state: ReaderState,
}

impl BufferReader {
	pub fn new() -> Self {
		Self {
			internal_buffer: Vec::new(),
			state: ReaderState::default(),
		}
	}

	pub fn push_bytes(&mut self, bytes: &[u8]) {
		self.internal_buffer.extend_from_slice(bytes);
	}
}
impl<'a> BufferReaderRef<'a> {
	pub fn new(buffer: &'a Vec<u8>) -> Self {
		Self {
			internal_buffer: buffer,
			state: ReaderState::default(),
		}
	}
}

type BaseIndex = usize;
struct Take<'buffer> {
	slice: &'buffer [u8],
	base_index: BaseIndex,
}

pub trait BufferReaderTakes: BufferReaderGetters {
	fn take_line(&mut self) -> Option<Take> {
		while self.im_state().interruptible_read_head < self.im_buf().len() {
			self.im_state_mut().interruptible_read_head += 1;
			let line = &self.im_buf()
				[self.im_state().bytes_taken..self.im_state().interruptible_read_head];

			match line.strip_suffix(b"\n") {
				Some(line) => {
					let line = line.strip_suffix(b"\r")
						.unwrap_or(line);

					let base_index = self.im_state().bytes_taken;
					self.im_state_mut().bytes_taken = self.im_state().interruptible_read_head;
					return Some(Take {
						slice: line,
						base_index,
					});
				}
				None => continue
			}
		}
		None
	}

	fn take_until(&mut self, sequence: &[u8]) -> Option<Take> {
		while self.im_state().interruptible_read_head < self.im_buf().len() {
			let slice = &self.im_buf()
				[self.im_state().bytes_taken..self.im_state().interruptible_read_head];

			if let Some(slice) = slice.strip_suffix(sequence) {
				let base_index = self.im_state().bytes_taken;
				self.im_state_mut().bytes_taken = self.im_state().interruptible_read_head;
				return Some(Take {
					slice,
					base_index,
				});
			}
		}
		None
	}

	fn take_exact(&mut self, size: usize) -> Option<Take> {
		if self.im_buf().len() <= self.im_state().bytes_taken + size {
			return None;
		}

		let base_index = self.im_state().bytes_taken;
		self.im_state_mut().bytes_taken += size;
		self.reset_read_head();
		Some(Take {
			slice: &self.im_buf()
				[base_index..self.im_state().bytes_taken],
			base_index,
		})
	}

	fn reset_read_head(&mut self) {
		self.im_state_mut().interruptible_read_head = self.im_state().bytes_taken;
	}

	fn bytes_remaining(&self) -> usize {
		self.im_buf().len() - self.im_state().bytes_taken
	}
}
impl BufferReaderTakes for BufferReader {}
impl BufferReaderTakes for BufferReaderRef<'_> {}

trait BufferReaderGetters {
	fn im_buf(&self) -> &[u8];
	fn im_state(&self) -> &ReaderState;
	fn im_state_mut(&mut self) -> &mut ReaderState;
}

impl BufferReaderGetters for BufferReader {
	fn im_buf(&self) -> &[u8] {
		&self.internal_buffer
	}

	fn im_state(&self) -> &ReaderState {
		&self.state
	}

	fn im_state_mut(&mut self) -> &mut ReaderState {
		&mut self.state
	}
}

impl BufferReaderGetters for BufferReaderRef<'_> {
	fn im_buf(&self) -> &[u8] {
		self.internal_buffer
	}

	fn im_state(&self) -> &ReaderState {
		&self.state
	}

	fn im_state_mut(&mut self) -> &mut ReaderState {
		&mut self.state
	}
}

impl Write for BufferReader {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.internal_buffer.write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.internal_buffer.flush()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn take_lines() {
		let mut buffer = BufferReader::new();

		buffer.write_all("line 1".as_bytes()).unwrap();
		assert!(buffer.take_line().is_none());

		buffer.write_all(" and more\r\nline 2\nline 3\r\n".as_bytes()).unwrap();
		buffer.take_line().and_then(|take| take.slice == b"");
		assert!(match buffer.take_line() {
			None => false,
			Some(_) => {}
		});
		assert!(buffer.take_line(), Some("line 1 and more".as_bytes()));
		assert!(buffer.take_line(), Some("line 2".as_bytes()), "without CR character");
		assert!(buffer.take_line(), Some("line 3".as_bytes()));
	}

	#[test]
	fn take_until() {
		let mut buffer = BufferReader::new();
		let sequence = "--sequence".as_bytes();
		buffer.write_all("some content".as_bytes()).unwrap();
		assert!(buffer.take_until(sequence).is_none());
		buffer.write_all("\r\n--".as_bytes()).unwrap();
		assert!(buffer.take_until(sequence).is_none());
		buffer.write_all("sequence".as_bytes()).unwrap();
		assert!(match buffer.take_until(sequence) {
			None => false,
			Some(take) => {
				take.slice == "some content\r\n".as_bytes()
			}
		});

		buffer.write_all("another content\r\n--sequence".as_bytes()).unwrap();
		assert!(match buffer.take_until(sequence) {
			None => false,
			Some(take) => {
				take.slice == "another content\r\n".as_bytes()
			}
		});
	}

	#[test]
	fn take_exact() {
		let mut buffer = BufferReader::new();
		buffer.write_all("data".as_bytes()).unwrap();
		assert!(buffer.take_exact(4), Some("data".as_bytes()));

		let mut buffer = BufferReader::new();

		buffer.write_all("data".as_bytes()).unwrap();
		assert!(buffer.take_exact(0), Some([].as_slice()));
		assert!(buffer.take_exact(10), None);
		assert!(buffer.take_exact(4), Some("data".as_bytes()));
		assert!(buffer.take_exact(10), None);

		buffer.write_all("long".as_bytes()).unwrap();
		assert!(buffer.take_exact(9), None);
		buffer.write_all("sword".as_bytes()).unwrap();
		assert!(buffer.take_exact(9), Some("longsword".as_bytes()));
	}
}
