#[derive(PartialEq, Debug, Default)]
enum ReadState {
	#[default]
	Ready,
	ReadLine,
	ReadLineFinished,
	ReadUntil(Vec<u8>),
	ReadUntilFinished,
	ReadExact(usize),
	ReadExactFinished,
}

use std::ops::Add;
use ReadState::*;
use crate::proto::internal::endline;

#[derive(Default)]
pub struct BufferReader {
	internal_buffer: Vec<u8>,
	read_head: usize,

	read_start: usize,
	read_end: usize,

	read_state: ReadState,
}

enum LineEnding {
	LF,
	CRLF,
}

impl BufferReader {
	pub fn append(&mut self, data: &[u8]) {
		self.internal_buffer.extend_from_slice(data);
	}

	pub fn advance(&mut self) -> bool {
		match &self.read_state {
			Ready
			| ReadLineFinished
			| ReadUntilFinished
			| ReadExactFinished
			=> true,

			ReadLine => {
				while self.read_head < self.internal_buffer.len() {
					if self.internal_buffer[self.read_head] == b'\n' {
						let line = endline::strip_last_endl(
							&self.internal_buffer[self.read_start..=self.read_head]
						);
						self.read_end = self.read_start + line.len();
						self.read_state = ReadLineFinished;
						return true
					}
					self.read_head += 1;
				}
				false
			}

			ReadUntil(sequence) => {
				while self.read_head < self.internal_buffer.len() {
					if self.internal_buffer[0..=self.read_head]
						.ends_with(sequence.as_slice()) {
						self.read_head += 1;
						self.read_end = self.read_head.checked_sub(sequence.len())
							.expect("invalid head position");
						self.read_state = ReadUntilFinished;
						return true;
					}
					self.read_head += 1;
				}
				false
			}

			ReadExact(size) => {
				while self.read_head < self.internal_buffer.len() {
					if (self.read_head + 1) - self.read_start == *size {
						self.read_state = ReadExactFinished;
						self.read_head += 1;
						self.read_end = self.read_head;
						return true;
					}
					self.read_head += 1;
				}
				false
			}
		}
	}

	pub fn read_line(&mut self) -> Option<&[u8]> {
		match self.read_state {
			Ready => {
				self.read_start = self.read_head;
				self.read_state = ReadLine;
				None
			}
			ReadLine => None,
			ReadLineFinished => {
				assert!(self.read_start <= self.read_end,
						"read_start is bigger than read_end");
				self.read_state = Ready;
				self.read_head += 1;
				Some(&self.internal_buffer
					[self.read_start..self.read_end])
			}
			_ => panic!("invalid operation; \
			tried to read_line when state was {:?}", self.read_state)
		}
	}

	pub fn read_until(&mut self, sequence: &[u8]) -> Option<&[u8]> {
		match self.read_state {
			Ready => {
				self.read_start = self.read_head;
				self.read_state = ReadUntil(sequence.to_vec());
				None
			}
			ReadUntil(_) => None,
			ReadUntilFinished => {
				assert!(self.read_start <= self.read_end,
						"read_start is bigger than read_end");
				self.read_state = Ready;
				Some(&self.internal_buffer
					[self.read_start..self.read_end])
			}
			_ => panic!("invalid operation; \
			tried to read_until when state was {:?}", self.read_state)
		}
	}

	pub fn read_exact(&mut self, size: usize) -> Option<&[u8]> {
		match self.read_state {
			Ready => {
				if size == 0 {
					return Some(&[])
				}

				self.read_start = self.read_head;
				self.read_state = ReadExact(size);
				None
			}
			ReadExact(_) => None,
			ReadExactFinished => {
				assert!(self.read_start <= self.read_end,
						"read_start is bigger than read_end");
				self.read_state = Ready;
				Some(&self.internal_buffer
					[self.read_start..self.read_end])
			}
			_ => panic!("invalid operation; \
			tried to read_exact when state was {:?}", self.read_state)
		}
	}

	pub fn get_head_idx(&self) -> usize {
		self.read_head
	}

	pub fn len(&self) -> usize {
		self.internal_buffer.len()
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn read_state_default() {
		let read_state = ReadState::default();
		assert_eq!(read_state, Ready);
	}

	#[test]
	fn advance_on_empty_buffer() {
		let mut buffer = BufferReader::default();
		assert_eq!(buffer.read_state, Ready);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), false);
	}

	#[test]
	fn simple_read_lines() {
		let mut buffer = BufferReader::default();
		assert_eq!(buffer.read_state, Ready);

		buffer.append("line 1\r\nline 2\r\n".as_bytes());
		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, ReadLineFinished);
		assert_eq!(buffer.read_line(), Some("line 1".as_bytes()),
				   "reading first line");
		assert_eq!(buffer.read_state, Ready);
		assert_eq!(buffer.read_head, 8,
				   "expecting head to be just after the first line");

		// unnecessary sanity check
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Ready);
		//

		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, ReadLineFinished);
		assert_eq!(buffer.read_line(), Some("line 2".as_bytes()),
				   "reading second line");

		assert_eq!(buffer.read_state, Ready);
	}

	#[test]
	fn simple_read_lines_simulate_async_buffer() {
		let mut buffer = BufferReader::default();
		buffer.append("  Lorem ipsum  \r\n".as_bytes());
		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, ReadLineFinished);
		assert_eq!(buffer.read_line(), Some("  Lorem ipsum  ".as_bytes()));

		assert_eq!(buffer.read_head, 17,
				   "head position after reading the 1st line should be after \\n, \
				   at the end of the current buffer ie. at index 17");

		buffer.append("  dolor sit amet".as_bytes());
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Ready);

		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), false);
		assert_eq!(buffer.read_state, ReadLine);

		buffer.append("\r".as_bytes());
		assert_eq!(buffer.advance(), false);
		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);

		buffer.append("\n".as_bytes());
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, ReadLineFinished);
		assert_eq!(buffer.read_line(), Some("  dolor sit amet".as_bytes()),
				   "extraction of the second line, with only the CRLF trimmed out");

		assert_eq!(buffer.read_head, 35,
				   "head position after reading the 2nd line should be after \\n, \
				   at index 35");

		buffer.append("\n".as_bytes());
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Ready);

		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_line(), Some("".as_bytes()),
				   "pushed a CRLF immediately after the last read_line \
				   and expecting an empty line");

		buffer.append("\r\n".as_bytes());
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Ready);

		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_line(), Some("".as_bytes()),
				   "expecting an empty line again, sanity check");

		assert_eq!(buffer.read_state, Ready);
	}

	#[test]
	fn read_until_on_simple_buffer() {
		let mut buffer = BufferReader::default();
		buffer.append("Lorem ipsum ===+ dolor sit amet==== after boundary".as_bytes());
		assert_eq!(buffer.read_until(b"===="), None);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_until(b"===="),
				   Some(b"Lorem ipsum ===+ dolor sit amet".as_slice()),
				   "trying to read all bytes until the '====' boundary");
		assert_eq!(buffer.read_state, Ready);
		assert_eq!(buffer.read_head, 35, "head is after the end of the boundary");
	}

	#[test]
	fn read_exact_zero() {
		let mut buffer = BufferReader::default();
		buffer.append(b"1234567890");
		assert_eq!(buffer.read_exact(0), Some(b"".as_slice()));
		assert_eq!(buffer.read_head, 0);
	}

	#[test]
	fn read_exact_simple_buffer() {
		let mut buffer = BufferReader::default();
		assert_eq!(buffer.read_exact(5), None);
		buffer.append(b"1234567890");

		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_exact(5), Some(b"12345".as_slice()));
		assert_eq!(buffer.read_head, 5);
	}

	#[test]
	fn read_exact_simulate_async_buffer() {
		let mut buffer = BufferReader::default();
		assert_eq!(buffer.read_exact(10), None);
		assert_eq!(buffer.read_state, ReadExact(10));
		buffer.append(&[1, 2, 3, 4]);

		assert_eq!(buffer.advance(), false);
		assert_eq!(buffer.read_exact(10), None);

		buffer.append(&[5, 6, 7, 8, 9, 10]);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_exact(10),
				   Some([1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10].as_slice()));

		assert_eq!(buffer.read_state, Ready);
	}
}
