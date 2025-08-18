use std::io::Write;

#[derive(Default)]
pub struct BufferReaderOwned {
	internal_buffer: Vec<u8>,
	meta: BufferReaderMeta
}

pub struct BufferReaderRef<'a> {
	internal_buffer: &'a [u8],
	meta: BufferReaderMeta
}

impl<'a> BufferReaderRef<'a> {
	pub fn new(wrapped_buffer: &'a [u8]) -> Self {
		Self {
			internal_buffer: wrapped_buffer,
			meta: BufferReaderMeta::default()
		}
	}
}

#[derive(Default, Debug)]
struct BufferReaderMeta {
	read_head: usize,
	bytes_taken: usize,
}

enum LineEnding {
	LF,
	CRLF,
}

impl Write for BufferReaderOwned {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.internal_buffer.extend_from_slice(buf);
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

impl BufferReaderOwned {
	pub fn take_line(&mut self) -> Option<&[u8]> {
		self.meta.take_line(self.internal_buffer.as_slice())
	}

	pub fn take_until(&mut self, sequence: &[u8]) -> Option<&[u8]> {
		self.meta.take_until(self.internal_buffer.as_slice(), sequence)
	}

	pub fn take_exact(&mut self, n_bytes: usize) -> Option<&[u8]> {
		self.meta.take_exact(self.internal_buffer.as_slice(), n_bytes)
	}

	pub fn take_all(&mut self) -> Option<&[u8]> {
		self.meta.take_all(self.internal_buffer.as_slice())
	}
}

impl BufferReaderRef<'_> {
	pub fn take_line(&mut self) -> Option<&[u8]> {
		self.meta.take_line(self.internal_buffer)
	}

	pub fn take_until(&mut self, sequence: &[u8]) -> Option<&[u8]> {
		self.meta.take_until(self.internal_buffer, sequence)
	}

	pub fn take_exact(&mut self, n_bytes: usize) -> Option<&[u8]> {
		self.meta.take_exact(self.internal_buffer, n_bytes)
	}

	// todo: make it not return Option
	pub fn take_all(&mut self) -> Option<&[u8]> {
		self.meta.take_all(self.internal_buffer)
	}
}

impl<'a> BufferReaderMeta {
	fn take_line(&mut self, buffer: &'a [u8]) -> Option<&'a [u8]> {
		while self.read_head < buffer.len() {
			let line = &buffer[self.bytes_taken..=self.read_head];
			self.read_head += 1;

			if let Some(line) = line.strip_suffix(b"\n") {
				let line = line.strip_suffix(b"\r").unwrap_or(line);

				self.bytes_taken = self.read_head;
				return Some(line);
			}
		}

		None
	}

	pub fn take_until(&mut self, buffer: &'a [u8], sequence: &[u8]) -> Option<&'a [u8]> {
		while self.read_head < buffer.len() {
			let line = &buffer[self.bytes_taken..=self.read_head];
			self.read_head += 1;

			if let Some(line) = line.strip_suffix(sequence) {
				self.bytes_taken = self.read_head;
				return Some(line);
			}
		}

		None
	}

	pub fn take_exact(&mut self, buffer: &'a [u8], n_bytes: usize) -> Option<&'a [u8]> {
		if n_bytes == 0 {
			return Some([].as_slice());
		}

		let data = buffer.get(
			self.bytes_taken..self.bytes_taken + n_bytes
		)?;

		self.bytes_taken += n_bytes;
		self.read_head = self.bytes_taken;
		Some(data)
	}

	pub fn take_all(&mut self, buffer: &'a [u8]) -> Option<&'a [u8]> {
		let tmp = &buffer[self.bytes_taken..];
		self.bytes_taken = buffer.len();
		self.read_head = buffer.len();
		Some(tmp)
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn take_lines() {
		let mut buffer = BufferReaderOwned::default();

		buffer.write_all("line 1".as_bytes()).unwrap();
		assert_eq!(buffer.take_line(), None);

		buffer.write_all(" and more\r\nline 2\nline 3\r\n".as_bytes()).unwrap();
		assert_eq!(buffer.take_line(), Some("line 1 and more".as_bytes()));
		assert_eq!(buffer.take_line(), Some("line 2".as_bytes()), "without CR character");
		assert_eq!(buffer.take_line(), Some("line 3".as_bytes()));
	}

	#[test]
	fn take_until() {
		let mut buffer = BufferReaderOwned::default();
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
		let mut buffer = BufferReaderOwned::default();
		buffer.write_all("data".as_bytes()).unwrap();
		assert_eq!(buffer.take_exact(4), Some("data".as_bytes()));

		let mut buffer = BufferReaderOwned::default();

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
