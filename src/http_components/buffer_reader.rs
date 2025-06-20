#[derive(PartialEq, Debug, Default)]
enum ReadState {
	#[default]
	Ready,
	SkipOne,
	SkipSpaces,
	ReadToken,
	ReadLine,
	ReadBlob(usize),
	Finished,
}
use ReadState::*;

#[derive(PartialEq, Debug)]
pub enum AdvanceResult {
	Done,
	Continue,
}
use AdvanceResult::*;

#[derive(Default)]
pub struct BufferReader {
	internal_buffer: Vec<u8>,
	read_head: usize,
	read_start: usize,

	read_state: ReadState,
}

#[derive(Debug)]
struct BufferIterationData {
	current_byte: u8,
	read_start: usize,
	read_head: usize,
}

impl BufferReader {
	pub fn append(&mut self, data: &[u8]) {
		self.internal_buffer.extend_from_slice(data);
	}

	fn iterate_on_buffer<F>(&mut self, mut f: F) -> AdvanceResult
	where
		F: FnMut(BufferIterationData) -> AdvanceResult,
	{
		while self.read_head < self.internal_buffer.len() {
			match f(
				BufferIterationData {
					current_byte: self.internal_buffer[self.read_head],
					read_start: self.read_start,
					read_head: self.read_head,
				}
			) {
				Done => return Done,
				Continue => {}
			};
			self.read_head += 1;
		};
		Continue
	}

	pub fn advance(&mut self) -> bool {
		match self.read_state {
			Ready | Finished => true,
			SkipOne => {
				if self.read_head + 1 < self.internal_buffer.len() {
					self.read_head += 1;
					self.read_state = Ready;
					true
				} else {
					false
				}
			}
			ReadBlob(blob_size) => {
				match self.iterate_on_buffer(|data| {
					if (data.read_head + 1) - data.read_start == blob_size {
						Done
					} else {
						Continue
					}
				}) {
					Done => {
						self.read_state = Finished;
						true
					}
					Continue => false
				}
			}
			_ => {
				match self.read_state {
					SkipSpaces => {
						match self.iterate_on_buffer(|data| {
							return if data.current_byte.is_ascii_whitespace() {
								// todo: error checking - illegal whitespace bytes
								Continue
							} else {
								Done
							};
						}) {
							Done => {
								self.read_state = Ready;
								true
							}
							Continue => false,
						}
					}
					ReadToken => {
						match self.iterate_on_buffer(|data| {
							if data.current_byte.is_ascii_whitespace() {
								Done
							} else {
								Continue
							}
						}) {
							Done => {
								self.read_state = Finished;
								true
							}
							Continue => false
						}
					}
					ReadLine => {
						match self.iterate_on_buffer(|data| {
							if data.current_byte == b'\n' {
								Done
							} else {
								Continue
							}
						}) {
							Done => {
								self.read_state = Finished;
								true
							}
							Continue => false
						}
					}

					Ready | Finished | ReadBlob(_) | SkipOne => unreachable!()
				}
			}
		}
	}

	pub fn skip_spaces(&mut self) {
		match self.read_state {
			Ready | Finished => {
				self.read_state = SkipSpaces;
			}
			_ => panic!("invalid state")
		}
	}

	fn set_read_state(&mut self, state: ReadState) {
		self.read_state = state;
		assert!(self.read_head == 0 || self.read_start != self.read_head,
				"invalid head state: attempted to read on the same position twice");
		self.read_start = self.read_head;
	}

	pub fn read_token(&mut self) -> Option<&[u8]> {
		match self.read_state {
			ReadToken => None,
			Ready => {
				self.set_read_state(ReadToken);
				None
			}
			Finished => {
				self.read_state = Ready;
				self.internal_buffer
					.get(self.read_start..self.read_head)
			}
			_ => panic!("invalid state")
		}
	}

	pub fn read_blob(&mut self, blob_size: usize) -> Option<&[u8]> {
		match self.read_state {
			ReadBlob(_) => None,
			Ready => {
				self.set_read_state(ReadBlob(blob_size));
				None
			}
			Finished => {
				self.read_state = SkipOne;
				self.internal_buffer
					.get(self.read_start..=self.read_head)
			}
			_ => panic!("invalid state")
		}
	}

	pub fn read_line(&mut self) -> Option<&[u8]> {
		match self.read_state {
			ReadLine => None,
			Ready => {
				println!("read_line: start_pos={}", self.read_start);
				self.set_read_state(ReadLine);
				None
			}
			Finished => {
				self.read_state = SkipOne;
				let line = &self.internal_buffer
					[self.read_start..self.read_head];
				println!("read_line finished: line={:?}, start={}, head={}", line, self.read_start, self.read_head);
				if line.last() == Some(&b'\r') {
					Some(&line[..line.len() - 1])
				} else {
					Some(line)
				}
			}
			_ => panic!("invalid state")
		}
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
		assert_eq!(buffer.read_token(), None);
		assert_eq!(buffer.advance(), false);
	}

	#[test]
	fn simple_read_tokens() {
		let mut buffer = BufferReader::default();
		assert_eq!(buffer.read_state, Ready);

		buffer.append("token1 token2 ".as_bytes());
		assert_eq!(buffer.read_token(), None);
		assert_eq!(buffer.read_state, ReadToken);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Finished);
		assert_eq!(buffer.read_token(), Some("token1".as_bytes()), "read_token != token1");
		assert_eq!(buffer.read_state, Ready);

		buffer.skip_spaces();
		assert_eq!(buffer.read_state, SkipSpaces);
		assert!(buffer.advance());
		assert_eq!(buffer.read_state, Ready);

		assert_eq!(buffer.read_token(), None);
		assert_eq!(buffer.read_state, ReadToken);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Finished);
		assert_eq!(buffer.read_token(), Some("token2".as_bytes()));
		assert_eq!(buffer.read_state, Ready);
	}

	#[test]
	fn read_tokens_partial() {
		let mut buffer = BufferReader::default();

		buffer.append("tok".as_bytes());
		assert_eq!(buffer.read_state, Ready);
		assert_eq!(buffer.read_token(), None);
		assert_eq!(buffer.read_state, ReadToken);
		assert_eq!(buffer.advance(), false);
		assert_eq!(buffer.read_state, ReadToken);

		buffer.append("en ".as_bytes());
		assert_eq!(buffer.read_token(), None);
		assert_eq!(buffer.read_state, ReadToken);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Finished);
		assert_eq!(buffer.read_token(), Some("token".as_bytes()));
		assert_eq!(buffer.read_state, Ready);
	}

	#[test]
	fn simple_read_blob() {
		let mut buffer = BufferReader::default();
		assert_eq!(buffer.read_blob(10), None);
		assert_eq!(buffer.read_state, ReadBlob(10));
		buffer.append(&[1, 2, 3, 4]);

		assert_eq!(buffer.advance(), false);
		assert_eq!(buffer.read_blob(10), None);

		buffer.append(&[5, 6, 7, 8, 9, 10]);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_blob(10),
				   Some([1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10].as_slice()));
	}

	#[test]
	fn simple_read_line() {
		let mut buffer = BufferReader::default();
		buffer.append("line\r\nanother line\r\n".as_bytes());
		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Finished);
		assert_eq!(buffer.read_line(), Some("line".as_bytes()));
		assert_eq!(buffer.advance(), true);

		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Finished);
		assert_eq!(buffer.read_line(), Some("another line".as_bytes()));
	}

	#[test]
	fn simple_read_line_2() {
		let mut buffer = BufferReader::default();
		buffer.append("  Lorem ipsum  \r\n".as_bytes());
		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Finished);
		assert_eq!(buffer.read_line(), Some("  Lorem ipsum  ".as_bytes()));
		assert_eq!(buffer.read_state, SkipOne);

		assert_eq!(buffer.read_head, 16);
		assert_eq!(buffer.advance(), false);
		assert_eq!(buffer.read_state, SkipOne);
		assert_eq!(buffer.read_head, 16);

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
		assert_eq!(buffer.read_state, Finished);
		assert_eq!(buffer.read_line(), Some("  dolor sit amet".as_bytes()));

		buffer.append("\n".as_bytes());
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Ready);

		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_line(), Some("".as_bytes()));

		buffer.append("\r\n".as_bytes());
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_state, Ready);

		assert_eq!(buffer.read_line(), None);
		assert_eq!(buffer.read_state, ReadLine);
		assert_eq!(buffer.advance(), true);
		assert_eq!(buffer.read_line(), Some("".as_bytes()));
	}

	#[test]
	fn complex_read_simulation() {
		let mut buffer = BufferReader::default();

		let appends = [
			"GET".as_bytes(),
			" ".as_bytes(),
			" ".as_bytes(),
		];
	}

}
