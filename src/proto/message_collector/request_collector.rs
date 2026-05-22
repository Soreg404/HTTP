use crate::proto::consts::{Method, Version};
use crate::proto::message_collector::{CollectError, MessageCollector, MessageType};
use crate::proto::rdx::Rdx;
use crate::proto::state_reader::Poll;
use crate::proto::url::UrlInfo;
use std::path::Path;

pub struct RequestCollector {
	buffer: Vec<u8>,
	message_collector: MessageCollector,
	request_basic: RequestBasic,
}

struct RequestBasic {
	method: Method,
	url: UrlInfo,
}

impl RequestCollector {
	pub fn new() -> Self {
		Self {
			buffer: vec![],
			message_collector: MessageCollector::new(MessageType::Request),
			request_basic: RequestBasic {
				method: Method::UNKNOWN,
				url: Default::default(),
			},
		}
	}

	pub fn is_finished(&self) -> Option<Result<(), CollectError>> {
		self.message_collector.finished
	}

	pub fn push_bytes(&mut self, bytes: &[u8]) {
		self.buffer.extend_from_slice(bytes);

		match self.message_collector.first_line(&self.buffer).clone() {
			None => {}
			Some(Poll::Pending) => return,
			Some(Poll::Ready(line)) => {
				let r = match self
					.request_basic
					.parse_request_line(
						line.get(&self.buffer),
						&mut self.message_collector.version
					) {
					Ok(()) => None,
					Err(e) => Some(Err(e.clone()))
				};
				self.message_collector.pass_first_line_parse_result(r);
			}
		};


		self.message_collector.process_buffer(&self.buffer);
	}
}

impl RequestBasic {
	fn parse_request_line(&mut self, line: &[u8], version: &mut Version)
						  -> Result<(), CollectError> {
		let mut i = 0;
		let mut sp_idx = None::<usize>;
		while i < line.len() {

			// check valid char

			if line[i] == b' ' {
				sp_idx = Some(i);
				break;
			}
			i += 1;
		}
		self.method = match sp_idx {
			None => return Err(CollectError::TBD),
			Some(v) => {
				Method::from_bytes(&line[..v])
			}
		};
		i += 1;
		println!("dbg method: {:?}", self.method);

		let begin_idx = i;
		sp_idx = None;
		while i < line.len() {

			// check another other valid (in url) char

			if line[i] == b' ' {
				sp_idx = Some(i);
				break;
			}
			i += 1;
		}
		self.url = match sp_idx {
			None => return Err(CollectError::TBD),
			Some(v) => {
				let url_bytes = &line[begin_idx..v];
				println!("dbg url_bytes: {:?}", String::from_utf8_lossy(url_bytes));
				match UrlInfo::parse_bytes(url_bytes) {
					Err(()) => return Err(CollectError::InvalidUrl),
					Ok(v) => {
						println!("url_info:");
						println!("path: {:?}", String::from_utf8_lossy(v.path.get(url_bytes)));
						println!("query: {:?}",
								 v.query_string.map(
									 |v| String::from_utf8_lossy(v.get(url_bytes))));
						println!("frag: {:?}",
								 v.fragment.map(
									 |v| String::from_utf8_lossy(v.get(url_bytes))));
						v
					}
				}
			}
		};
		i += 1;

		let vs = &line[i..];
		// if vs.len() != 8 || vs[6] != b'.' || !vs.starts_with(b"HTTP/")
		// 	|| !vs[5].is_ascii_digit() || !vs[7].is_ascii_digit() {
		// 	return Err(CollectError::TBD);
		// }
		// let hi = vs[5] - b'0';
		// let lo = vs[7] - b'0';
		*version = match Version::from_bytes(vs) {
			Err(()) => return Err(CollectError::InvalidVersion),
			Ok(v) => v
		};

		Ok(())
	}
}

fn parse_multipart_content_type_header(value: &[u8])
									   -> Result<Option<Rdx>, CollectError> {
	let mut i = 0;
	let err = Err(CollectError::InvalidContentTypeHeader);

	match get_word(value, &mut i, b';') {
		Err(()) => return err,
		Ok(word) => {
			if !word.eq_ignore_ascii_case(b"multipart/form-data") {
				return Ok(None);
			}
		}
	};

	match get_word(value, &mut i, b'=') {
		Err(()) => return err,
		Ok(word) => {
			if !word.eq_ignore_ascii_case(b"boundary") {
				return err;
			}
		}
	}

	skip_ws(value, &mut i);

	if value[i] == b'"' {
		i += 1;
		let word_start = i;
		while i < value.len() {
			// todo check valid in header string
			if value[i] == b'"' {
				break;
			}
			i += 1;
		}
		let boundary = Rdx::new(word_start, i);

		i += 1;

		skip_ws(value, &mut i);
		if i < value.len() {
			return err;
		}

		Ok(Some(boundary))
	} else {
		let word_start = i;

		while i < value.len() {
			// todo check valid outside header string
			if value[i] == b' ' {
				break;
			}
			i += 1;
		}

		let boundary = Rdx::new(word_start, i);

		skip_ws(value, &mut i);
		if i < value.len() {
			return err;
		}

		Ok(Some(boundary))
	}
}

fn skip_ws(value: &[u8], i: &mut usize) {
	while *i < value.len() {
		if value[*i] != b' ' {
			break;
		}
		*i += 1;
	}
}

fn get_word<'a>(value: &'a [u8], i: &mut usize, delim_byte: u8)
				-> Result<&'a [u8], ()> {
	skip_ws(value, i);

	let word_start = *i;
	while *i < value.len() {
		// todo check valid?
		if value[*i] == b' ' || value[*i] == delim_byte {
			break;
		}
		*i += 1;
	}
	let word_end = *i;

	skip_ws(value, i);

	if *i == value.len() || value[*i] != delim_byte {
		return Err(());
	}

	*i += 1;

	Ok(&value[word_start..word_end])
}


#[test]
fn test_content_type_header_parser() {
	use parse_multipart_content_type_header as p;
	let e = Err(CollectError::InvalidContentTypeHeader);
	println!("==================");
	assert_eq!(p(b"multipart/form-data"), e);
	println!("==================");
	assert_eq!(p(b"multipart/form-data; boundary"), e);
	println!("==================");
	assert_eq!(p(b"   multipart/form-data   ; boundary   =   HelloWorld   "),
			   Ok(Some(Rdx::new(42, 52))));
	println!("==================");
	assert_eq!(p(b"multipart/form-data; boundary=\"abc\""),
			   Ok(Some(Rdx::new(31, 34))));
}

impl RequestCollector {
	pub fn dump_first_attachment_data(&self) {
		match self.message_collector.attachments.first() {
			None => {
				println!("fuk u no attachments");
			}
			Some(a) => {
				println!("attachment: name={:?}, filename={:?}",
						 String::from_utf8_lossy(a.name.unwrap().get(&self.buffer)),
						 a.filename.map(|v| String::from_utf8_lossy(v.get(&self.buffer)))
				);
				println!("dumping to file...");

				let filename = match a.filename {
					None => b"file.png",
					Some(v) => v.get(&self.buffer)
				};
				let filename = String::from_utf8_lossy(filename).to_string();

				std::fs::write(Path::new(&filename), a.data.get(&self.buffer)).unwrap();
			}
		}
	}
}
