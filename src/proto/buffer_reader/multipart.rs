use super::{BufferReader, BufferReaderResult};

pub struct PartInfo {}


impl BufferReader {
	pub fn take_next_part<'a>(
		&mut self,
		buffer: &'a [u8],
		boundary: &[u8],
		length_limit: Option<usize>,
	) -> BufferReaderResult<PartInfo> {

		loop {

			if self.current_read_head == buffer.len() {
				return BufferReaderResult::NotEnoughBytes;
			}

			match length_limit {
				Some(v) => {
					if self.current_read_head >= v {
						panic!()
					}
				}
				None => {}
			}

			match buffer[..self.current_read_head].strip_suffix(b"\n") {
				Some(s) => {

				}
				None => {}
			}


			self.current_read_head += 1;
		}
	}
}

#[test]
fn take_next_part() {
	let sample = "skip\r\n--abc\r\npart1\r\n--abc\r\npart2\r\n--abc--\r\n";
}
