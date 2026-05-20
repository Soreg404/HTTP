use crate::consts::{Method, MimeType, Version};
use crate::proto::url::UrlMeta;

pub struct RequestCollector {
	buffer: Vec<u8>,
	collector_meta: CollectorMeta,
	collector_message_common: CollectorMessageCommon,
	collector_request_specific: CollectorRequestSpecific,
}

struct CollectorRequestSpecific {
	method: Method,
	url_rdx: Rdx,
	url_meta: UrlMeta,
}

struct CollectorMeta {
	finished: Option<Result<(), CollectError>>,
	stage: ProcStage,
	buf_head: usize,
	buf_base: usize,
}

struct CollectorMessageCommon {
	version: Version,

	headers: Vec<Header>,

	content_length: Option<usize>,
	boundary_rdx: Option<Rdx>,

	body_meta: Option<Rdx>,
	attachments: Vec<Attachment>,
}

struct Header {
	name: Rdx,
	value: Rdx,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Rdx {
	from: usize,
	to: usize,
}

impl Rdx {
	pub(crate) fn new(from: usize, to: usize) -> Self {
		Self { from, to }
	}
	fn with_base(self, base: usize) -> Self {
		Self {
			from: self.from + base,
			to: self.to + base,
		}
	}
	fn get<'a>(&self, from: &'a [u8]) -> &'a [u8] {
		&from[self.from..self.to]
	}
}

struct Attachment {
	name: Rdx,
	mime_type: MimeType,
}

#[derive(Copy, Clone, Debug)]
#[derive(PartialEq)]
pub enum CollectError {
	TBD,
	IllegalCharacter,
	UnknownMethod,
	InvalidUrl,
	InvalidVersion,
	InvalidContentTypeHeader,
}

#[derive(Debug, Copy, Clone)]
enum ProcStage {
	FirstLine,
	MainHeaders,
	PreBody,
	Body(ProcBody),
}

#[derive(Debug, Clone, Copy)]
enum ProcBody {
	Normal {
		length: usize,
	},
	Chunked,
	Multipart {
		init: bool,
		boundary_rdx: Rdx,
	},
}

enum TakeResult<T> {
	Ready(T),
	More,
}

impl RequestCollector {
	pub fn new() -> Self {
		Self {
			buffer: vec![],
			collector_meta: CollectorMeta {
				finished: None,
				stage: ProcStage::FirstLine,
				buf_head: 0,
				buf_base: 0,
			},
			collector_message_common: CollectorMessageCommon {
				version: Version::HTTP_0_9,
				headers: vec![],
				content_length: None,
				boundary_rdx: None,
				body_meta: None,
				attachments: vec![],
			},
			collector_request_specific: CollectorRequestSpecific {
				method: Method::UNKNOWN,
				url_rdx: Default::default(),
				url_meta: Default::default(),
			},
		}
	}

	pub fn push_bytes(&mut self, bytes: &[u8]) {
		self.buffer.extend_from_slice(bytes);

		let mut safety_switch = 0;

		loop {
			println!("push_bytes() loop");
			assert!(self.collector_meta.finished.is_none(),
					"forgor to return? stage: {:?}", self.collector_meta.stage);

			match self.collector_meta.stage.clone() {
				ProcStage::FirstLine => {
					println!("first line");
					match self.collector_meta.take_line(&self.buffer) {
						TakeResult::More => return,
						TakeResult::Ready(line) => {
							println!("ready ({:?})", String::from_utf8_lossy(&line));
							match self.collector_request_specific
								.parse_request_line(
									line,
									&mut self.collector_message_common.version,
								) {
								Err(e) => {
									self.collector_meta.finished = Some(Err(e));
									println!("err: {e:?}");
									return;
								}
								Ok(()) => {
									self.collector_meta.stage = ProcStage::MainHeaders;
									println!("ok");
								}
							}
						}
					}
				}
				ProcStage::MainHeaders => {
					println!("main headers");
					let idx_base = self.collector_meta.buf_base;
					match self.collector_meta.take_line(&self.buffer) {
						TakeResult::More => return,
						TakeResult::Ready(line) => {
							println!("ready ({:?})", String::from_utf8_lossy(&line));
							if line.trim_ascii().is_empty() {
								println!("empty header line");
								if !line.is_empty() {
									self.collector_meta.finished = Some(Err(CollectError::TBD));
									return;
								} else {
									self.collector_meta.stage = ProcStage::PreBody
								}
							} else {
								println!("non empty header line");
								match parse_header_line(line) {
									Err(e) => {
										self.collector_meta.finished = Some(Err(e));
										return;
									}
									Ok(header) => {
										let name = header.name.get(line);
										let value = header.value.get(line);
										println!(
											"dbg header: {:?}:{:?}",
											String::from_utf8_lossy(name),
											String::from_utf8_lossy(value),
										);

										let header_based = Header {
											name: header.name.with_base(idx_base),
											value: header.value.with_base(idx_base),
										};
										println!(
											"header based: {:?}:{:?}",
											String::from_utf8_lossy(
												header_based.name.get(&self.buffer)),
											String::from_utf8_lossy(
												header_based.value.get(&self.buffer)),
										);
										let value_base = header_based.value.from;

										self.collector_message_common.push_header(header_based);

										if name.eq_ignore_ascii_case(b"content-length") {
											match su8_to_dec(value) {
												Err(()) => {
													self.collector_meta.finished = Some(Err(CollectError::TBD));
													return;
												}
												Ok(v) => {
													self.collector_message_common.content_length = Some(v);
												}
											}
										} else if name
											.eq_ignore_ascii_case(b"content-type") {
											match parse_multipart_content_type_header(value) {
												Err(e) => {
													self.collector_meta.finished = Some(Err(e));
													return;
												}
												Ok(None) => {}
												Ok(Some(v)) => {
													self.collector_message_common
														.boundary_rdx = Some(v.with_base(value_base));
													println!("found boundary: {:?}",
															 String::from_utf8_lossy(self.collector_message_common
																 .boundary_rdx
																 .unwrap()
																 .get(&self.buffer)))
												}
											};
										}
									}
								};
							}
						}
					}
				}
				ProcStage::PreBody => {
					println!("pre-body");
					match self.collector_message_common.content_length {
						None => {
							self.collector_meta.finished = Some(Ok(()));
							return;
						}
						Some(v) => {
							match self.collector_message_common.boundary_rdx {
								None => {
									self.collector_meta.stage = ProcStage::Body(
										ProcBody::Normal { length: v }
									);
								}
								Some(rdx) => {
									self.collector_meta.stage = ProcStage::Body(
										ProcBody::Multipart {
											init: true,
											boundary_rdx: rdx,
										}
									)
								}
							}
						}
					}
				}

				ProcStage::Body(v) => match v {
					ProcBody::Normal { length } => {
						println!("body normal");
						if self.collector_meta.buf_base + length <= self.buffer.len() {
							let body_rdx = Rdx::new(
								self.collector_meta.buf_base,
								self.collector_meta.buf_base + length,
							);
							self.collector_meta.finished = Some(Ok(()));
							println!("finished: {:?}", String::from_utf8_lossy(
								body_rdx.get(&self.buffer)
							));
						}
						return;
					}
					ProcBody::Chunked => todo!(),
					ProcBody::Multipart {
						init,
						boundary_rdx,
					} => {
						let boundary = boundary_rdx.get(&self.buffer);

					}
				}
			}

			safety_switch += 1;
			if safety_switch > 10 {
				return;
			}
		}
	}

	pub fn is_finished(&self) -> Option<Result<(), CollectError>> {
		self.collector_meta.finished.clone()
	}
}

impl CollectorRequestSpecific {
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
		self.url_meta = match sp_idx {
			None => return Err(CollectError::TBD),
			Some(v) => {
				let url_bytes = &line[begin_idx..v];
				println!("dbg url_bytes: {:?}", String::from_utf8_lossy(url_bytes));
				match UrlMeta::parse_bytes(url_bytes) {
					Ok(v) => {
						self.url_rdx = Rdx::new(begin_idx, begin_idx + url_bytes.len());
						println!("url meta: {v:?}, rdx: {:?}", self.url_rdx);
						v
					}
					Err(e) => return Err(CollectError::InvalidUrl)
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

impl CollectorMeta {
	fn take_line<'a>(&mut self, buffer: &'a [u8]) -> TakeResult<&'a [u8]> {
		while self.buf_head < buffer.len() {
			match buffer[self.buf_base..=self.buf_head].strip_suffix(b"\n") {
				None => {}
				Some(s) => {
					let s = s.strip_suffix(b"\r").unwrap_or(s);

					self.buf_head += 1;
					self.buf_base = self.buf_head;
					return TakeResult::Ready(s);
				}
			}
			self.buf_head += 1;
		}

		TakeResult::More
	}
}

impl CollectorMessageCommon {
	fn push_header(&mut self, header: Header) {}
}

fn parse_header_line(line: &[u8]) -> Result<Header, CollectError> {
	enum Part {
		Name,
		Value,
	}
	let mut p = Part::Name;
	let mut i = 0;
	let mut colon_idx = 0;
	let mut value_first_byte = None::<usize>;
	let mut value_last_byte = 0;
	while i < line.len() {
		match p {
			Part::Name => {
				// todo: check valid chars in field name
				if line[i] == b':' {
					colon_idx = i;
					p = Part::Value;
					value_last_byte = i + 1;
				}
			}
			Part::Value => {
				// todo: check valid
				if line[i] != b' ' {
					if value_first_byte.is_none() {
						value_first_byte = Some(i);
					}
					value_last_byte = i;
				}
			}
		}
		i += 1;
	}

	Ok(
		Header {
			name: Rdx::new(0, colon_idx),
			value: Rdx::new(
				value_first_byte.unwrap_or(value_last_byte),
				value_last_byte + 1,
			),
		}
	)
}

fn su8_to_dec(s: &[u8]) -> Result<usize, ()> {
	let mut v = 0usize;
	for c in s {
		if !c.is_ascii_digit() {
			return Err(());
		}
		v = v * 10 + (c - b'0') as usize;
	}
	Ok(v)
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
