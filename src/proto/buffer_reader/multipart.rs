use super::{BufferReader, BufferReaderResult};
use crate::proto::parser::ParseError;

pub struct PartInfo<'a> {
	is_last: bool,
	slice: &'a [u8],
}


impl BufferReader {
	pub fn take_next_part<'a>(
		&mut self,
		buffer: &'a [u8],
		boundary: &[u8],
		length_limit: Option<usize>,
	) -> BufferReaderResult<Result<PartInfo<'a>, ParseError>> {
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

			match buffer[self.n_bytes_consumed..=self.current_read_head]
				.strip_suffix(b"\n") {
				None => {}
				Some(s) => {
					let s = s.strip_suffix(b"\r").unwrap_or(s);

					match s.strip_suffix(boundary) {
						None => {}
						Some(s) => {
							match s.strip_suffix(b"\n--") {
								None => {}
								Some(s) => {
									let s = s.strip_suffix(b"\r").unwrap_or(s);

									self.current_read_head += 1;
									self.n_bytes_consumed = self.current_read_head;
									return BufferReaderResult::Done(
										Ok(
											PartInfo {
												is_last: false,
												slice: s,
											}
										)
									);
								}
							}
						}
					}
				}
			}


			self.current_read_head += 1;
		}
	}
}

#[test]
fn take_next_part() {
	let s_skip = b"skiptext";
	let s_part1 = b"first part";
	let s_part2 = b"second part";
	let s_rn = b"\r\n";
	let s_dash = b"--";
	let s_boundary = b"abc";

	let boundary_len = s_rn.len() * 2 + s_dash.len() + s_boundary.len();

	let mut part_start = 0;

	let mut buf = Vec::<u8>::new();
	let mut rd = BufferReader::new();

	buf.extend_from_slice(s_skip);
	buf.extend_from_slice(s_rn);
	buf.extend_from_slice(s_dash);
	buf.extend_from_slice(s_boundary);

	// incomplete part, no \r\n after boundary
	match rd.take_next_part(&buf, s_boundary, None) {
		BufferReaderResult::NotEnoughBytes => {}
		BufferReaderResult::Done(_) => panic!()
	}

	buf.extend_from_slice(s_rn);

	// complete skip-text part
	match rd.take_next_part(&buf, s_boundary, None) {
		BufferReaderResult::NotEnoughBytes => panic!(),
		BufferReaderResult::Done(r) => match r {
			Err(_) => panic!(),
			Ok(info) => {
				assert_eq!(info.is_last, false);
				assert_eq!(info.slice, &buf[..s_skip.len()]);

				assert_eq!(rd.n_bytes_consumed, s_skip.len() + boundary_len);
				assert_eq!(rd.current_read_head, rd.n_bytes_consumed);
			}
		}
	}

	part_start = s_skip.len() + boundary_len;

	buf.extend_from_slice(s_part1);
	buf.extend_from_slice(s_rn);
	buf.extend_from_slice(s_dash);

	// incomplete part, missing boundary\r\n
	match rd.take_next_part(&buf, s_boundary, None) {
		BufferReaderResult::NotEnoughBytes => {}
		BufferReaderResult::Done(_) => panic!()
	}

	buf.extend_from_slice(s_boundary);
	buf.extend_from_slice(s_rn);

	// complete first part
	match rd.take_next_part(&buf, s_boundary, None) {
		BufferReaderResult::NotEnoughBytes => panic!(),
		BufferReaderResult::Done(r) => match r {
			Err(_) => panic!(),
			Ok(info) => {
				assert_eq!(info.is_last, false);
				assert_eq!(info.slice, &buf[part_start..s_part1.len()]);

				assert_eq!(rd.n_bytes_consumed, part_start + boundary_len);
			}
		}
	}
}
