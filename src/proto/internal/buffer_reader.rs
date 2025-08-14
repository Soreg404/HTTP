use std::io::Write;

#[derive(Default)]
pub struct BufferReader {
	internal_buffer: Vec<u8>,
	read_head: usize,
	bytes_taken: usize,
}

enum LineEnding {
	LF,
	CRLF,
}

impl Write for BufferReader {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.internal_buffer.extend_from_slice(buf);
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

impl BufferReader {
	pub fn take_line(&mut self) -> Option<&[u8]> {
		while self.read_head < self.internal_buffer.len() {
			let line = &self.internal_buffer[self.bytes_taken..=self.read_head];
			self.read_head += 1;

			if let Some(line) = line.strip_suffix(b"\n") {
				let line = line.strip_suffix(b"\r").unwrap_or(line);

				self.bytes_taken = self.read_head;
				return Some(line);
			}
		}

		None
	}

	pub fn take_until(&mut self, sequence: &[u8]) -> Option<&[u8]> {
		while self.read_head < self.internal_buffer.len() {
			let line = &self.internal_buffer[self.bytes_taken..=self.read_head];
			self.read_head += 1;

			if let Some(line) = line.strip_suffix(sequence) {
				self.bytes_taken = self.read_head;
				return Some(line);
			}
		}

		None
	}

	pub fn take_exact(&mut self, n_bytes: usize) -> Option<&[u8]> {
		if n_bytes == 0 {
			return Some([].as_slice());
		}

		let data = self.internal_buffer.get(
			self.bytes_taken..self.bytes_taken + n_bytes
		)?;

		self.bytes_taken += n_bytes;
		self.read_head = self.bytes_taken;
		Some(data)
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn take_lines() {
		let mut buffer = BufferReader::default();

		buffer.write_all("line 1".as_bytes()).unwrap();
		assert_eq!(buffer.take_line(), None);

		buffer.write_all(" and more\r\nline 2\nline 3\r\n".as_bytes()).unwrap();
		assert_eq!(buffer.take_line(), Some("line 1 and more".as_bytes()));
		assert_eq!(buffer.take_line(), Some("line 2".as_bytes()), "without CR character");
		assert_eq!(buffer.take_line(), Some("line 3".as_bytes()));
	}

	#[test]
	fn take_until() {
		let mut buffer = BufferReader::default();
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
		let mut buffer = BufferReader::default();
		buffer.write_all("data".as_bytes()).unwrap();
		assert_eq!(buffer.take_exact(4), Some("data".as_bytes()));

		let mut buffer = BufferReader::default();

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
