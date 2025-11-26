use std::io::Write;

#[derive(Default)]
struct ReaderState {
	interruptible_read_head: usize,
	bytes_taken: usize,
}

struct BufferReader {
	internal_buffer: Vec<u8>,
	state: ReaderState,
}

struct BufferReaderRef<'itl_buf> {
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
}
impl<'a> BufferReaderRef<'a> {
	pub fn new(buffer: &'a Vec<u8>) -> Self {
		Self {
			internal_buffer: buffer,
			state: ReaderState::default(),
		}
	}
}

struct TraitBase<'a> {
	internal_buffer: &'a [u8],
	state: &'a mut ReaderState,
}

impl BufferReaderTakes for BufferReader {
	fn base(&mut self) -> TraitBase {
		TraitBase {
			internal_buffer: &self.internal_buffer,
			state: &mut self.state,
		}
	}
}

impl BufferReaderTakes for BufferReaderRef<'_> {
	fn base(&mut self) -> TraitBase {
		TraitBase {
			internal_buffer: self.internal_buffer,
			state: &mut self.state,
		}
	}
}


trait BufferReaderTakes {
	fn base(&mut self) -> TraitBase;
	fn take_line(&mut self) -> Option<&[u8]> {
		let a = self.base();

		while a.state.interruptible_read_head < a.internal_buffer.len() {
			a.state.interruptible_read_head += 1;
			let line = &a.internal_buffer
				[a.state.bytes_taken..a.state.interruptible_read_head];

			match line.strip_suffix(b"\n") {
				Some(line) => {
					match line.strip_suffix(b"\r") {
						Some(line) => return Some(line),
						None => continue
					}
				}
				None => continue
			}
		}
		None
	}

	fn take_until(&mut self, sequence: &[u8]) -> Option<&[u8]> {
		let a = self.base();

		while a.state.interruptible_read_head < a.internal_buffer.len() {
			let slice = &a.internal_buffer
				[a.state.bytes_taken..a.state.interruptible_read_head];

			if slice.strip_suffix(sequence).is_some() {
				return Some(slice);
			}
		}
		None
	}

	fn take_exact(&mut self, size: usize) -> Option<&[u8]> {
		let a = self.base();

		if a.internal_buffer.len() <= a.state.bytes_taken + size {
			return None;
		}

		let old_bytes_taken = a.state.bytes_taken;
		a.state.bytes_taken += size;
		self.reset_read_head();

		Some(&self.base().internal_buffer
			[old_bytes_taken..old_bytes_taken + size])
	}

	fn reset_read_head(&mut self) {
		let a = self.base();
		a.state.interruptible_read_head = a.state.bytes_taken;
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
		assert_eq!(buffer.take_line(), None);

		buffer.write_all(" and more\r\nline 2\nline 3\r\n".as_bytes()).unwrap();
		assert_eq!(buffer.take_line(), Some("line 1 and more".as_bytes()));
		assert_eq!(buffer.take_line(), Some("line 2".as_bytes()), "without CR character");
		assert_eq!(buffer.take_line(), Some("line 3".as_bytes()));
	}

	#[test]
	fn take_until() {
		let mut buffer = BufferReader::new();
		let sequence = "--sequence".as_bytes();
		buffer.write_all("some content".as_bytes()).unwrap();
		assert_eq!(buffer.take_until(sequence), None);
		buffer.write_all("\r\n--".as_bytes()).unwrap();
		assert_eq!(buffer.take_until(sequence), None);
		buffer.write_all("sequence".as_bytes()).unwrap();
		assert_eq!(buffer.take_until(sequence), Some("some content\r\n".as_bytes()));

		buffer.write_all("another content\r\n--sequence".as_bytes()).unwrap();
		assert_eq!(buffer.take_until(sequence), Some("another content\r\n".as_bytes()));
	}

	#[test]
	fn take_exact() {
		let mut buffer = BufferReader::new();
		buffer.write_all("data".as_bytes()).unwrap();
		assert_eq!(buffer.take_exact(4), Some("data".as_bytes()));

		let mut buffer = BufferReader::new();

		buffer.write_all("data".as_bytes()).unwrap();
		assert_eq!(buffer.take_exact(0), Some([].as_slice()));
		assert_eq!(buffer.take_exact(10), None);
		assert_eq!(buffer.take_exact(4), Some("data".as_bytes()));
		assert_eq!(buffer.take_exact(10), None);

		buffer.write_all("long".as_bytes()).unwrap();
		assert_eq!(buffer.take_exact(9), None);
		buffer.write_all("sword".as_bytes()).unwrap();
		assert_eq!(buffer.take_exact(9), Some("longsword".as_bytes()));
	}
}
