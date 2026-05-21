use crate::consts::{Method, MimeType, Version};
use crate::proto::header_parser::HeaderParser;
use crate::proto::rdx::Rdx;
use crate::proto::state_reader::{Poll, StateReader};
use crate::proto::url::UrlMeta;

pub struct RequestCollector {
	buffer: Vec<u8>,
	collector_state: CollectorState,
	message_common: MessageCommon,
	request_basic: RequestBasic,
}

struct RequestBasic {
	method: Method,
	url_rdx: Rdx,
	url_meta: UrlMeta,
}

struct CollectorState {
	finished: Option<Result<(), CollectError>>,
	stage: ProcStage,
	state_reader: StateReader,
}

struct MessageCommon {
	version: Version,

	headers: Vec<TmpHeader>,

	content_length: Option<usize>,
	boundary_rdx: Option<Rdx>,

	body: Option<Rdx>,
	attachments: Vec<Attachment>,

	current_attachment: Attachment,
}

pub struct TmpHeader {
	name: Rdx,
	value: Rdx,
}

#[derive(Debug, Clone)]
struct Attachment {
	name: Rdx,
	filename: Option<Rdx>,
	mime_type: MimeType,
	data: Rdx,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CollectError {
	TBD,
	IllegalCharacter,
	UnknownMethod,
	InvalidUrl,
	InvalidVersion,
	InvalidContentTypeHeader,
	InvalidHeader,
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
	Normal,
	Chunked,
	Multipart(ProcMultipart),
}

#[derive(Clone, Copy, Debug)]
enum ProcMultipart {
	Init,
	Headers,
	Data,
}

impl RequestCollector {
	pub fn new() -> Self {
		Self {
			buffer: vec![],
			collector_state: CollectorState {
				finished: None,
				stage: ProcStage::FirstLine,
				state_reader: StateReader::new(),
			},
			message_common: MessageCommon {
				version: Version::HTTP_0_9,
				headers: vec![],
				content_length: None,
				boundary_rdx: None,
				attachments: vec![],
				body: None,
				current_attachment: Attachment {
					name: Default::default(),
					filename: None,
					mime_type: MimeType::Unspecified,
					data: Default::default(),
				},
			},
			request_basic: RequestBasic {
				method: Method::UNKNOWN,
				url_rdx: Default::default(),
				url_meta: Default::default(),
			},
		}
	}

	pub fn push_bytes(&mut self, bytes: &[u8]) {
		self.buffer.extend_from_slice(bytes);

		match self.collector_state.stage.clone() {
			ProcStage::FirstLine => {
				match self.collector_state.state_reader.take_line(&self.buffer) {
					Poll::Pending => return,
					Poll::Ready(line) => {
						match self.request_basic
							.parse_request_line(
								line.get(&self.buffer),
								&mut self.message_common.version,
							) {
							Err(e) => {
								self.collector_state.finished = Some(Err(e));
							}
							Ok(()) => {
								self.collector_state.stage = ProcStage::MainHeaders;

								self.message_common.process_buffer(
									&mut self.buffer,
									&mut self.collector_state,
								);
							}
						}
					}
				}
			}
			_ => {
				self.message_common.process_buffer(
					&mut self.buffer,
					&mut self.collector_state,
				)
			}
		}
	}

	pub fn is_finished(&self) -> Option<Result<(), CollectError>> {
		self.collector_state.finished.clone()
	}
}

impl MessageCommon {
	pub fn process_buffer(
		&mut self,
		buffer: &mut [u8],
		collector_state: &mut CollectorState,
	) {
		loop {
			match collector_state.stage.clone() {
				ProcStage::FirstLine => unreachable!(),
				ProcStage::MainHeaders => {
					match collector_state.state_reader.take_line(buffer) {
						Poll::Pending => return,
						Poll::Ready(line) => {
							self.process_header_line(
								buffer,
								line,
								collector_state,
							)
						}
					}
				}
				ProcStage::PreBody => {
					match self.content_length {
						None => {
							collector_state.finished = Some(Ok(()));
						}
						Some(_) => {
							match self.boundary_rdx {
								None => {
									collector_state.stage =
										ProcStage::Body(ProcBody::Normal);
								}
								Some(_) => {
									collector_state.stage =
										ProcStage::Body(
											ProcBody::Multipart(
												ProcMultipart::Init))
								}
							}
						}
					}
				}
				ProcStage::Body(ProcBody::Normal) => {
					let content_length = self.content_length.unwrap();

					let body_start = collector_state.state_reader.base;
					let body_end = body_start + content_length;

					if body_end > buffer.len() {
						return;
					}

					self.body = Some(Rdx::new(body_start, body_end));
					collector_state.finished = Some(Ok(()));

					return;
				}
				ProcStage::Body(ProcBody::Chunked) => todo!(),
				ProcStage::Body(ProcBody::Multipart(pm)) => match pm {
					ProcMultipart::Init => {
						println!("multipart init");
						let boundary = self.boundary_rdx.unwrap().get(buffer);
						println!("lookin for boundary: {:?}",
								 String::from_utf8_lossy(boundary));

						match collector_state.state_reader.take_attachment(buffer, boundary) {
							Poll::Pending => return,
							Poll::Ready(bi) => {
								println!("init found attachment, data: {:?}",
										 String::from_utf8_lossy(bi.data.get(buffer)));

								if bi.is_last {
									collector_state.finished = Some(Ok(()));
									return;
								}
								collector_state.stage =
									ProcStage::Body(
										ProcBody::Multipart(
											ProcMultipart::Headers));
							}
						};
					}
					ProcMultipart::Headers => {
						println!("multipart headers");

						let line = match collector_state
							.state_reader.take_line(buffer) {
							Poll::Pending => return,
							Poll::Ready(line) => line
						};

						println!("[multipart headers] line: {:?}",
								 String::from_utf8_lossy(line.get(buffer)));

						let line_bytes = line.get(buffer);
						if line_bytes.trim_ascii().is_empty() {
							if !line_bytes.is_empty() {
								collector_state.finished = Some(Err(CollectError::TBD));
								return;
							}
							collector_state.stage =
								ProcStage::Body(
									ProcBody::Multipart(
										ProcMultipart::Data));

							self.attachments.push(self.current_attachment.clone());

							println!("[multipart headers] empty line, pushing attachment: \n\
							{:?}", self.current_attachment.clone());

							continue
						}

						let mut hp = match HeaderParser::new(line_bytes) {
							Err(()) => {
								collector_state.finished = Some(Err(CollectError::InvalidHeader));
								return;
							}
							Ok(hp) => hp
						};

						let f_name = hp.field_name()
							.get(line_bytes);

						if f_name.eq_ignore_ascii_case(b"content-disposition") {
							println!("[multiart headers] parsing content-disposition");
							match hp.next_atom() {
								None => {
									collector_state.finished = Some(Err(CollectError::TBD));
									return;
								}
								Some(s) => {
									if !s.get(line_bytes)
										.eq_ignore_ascii_case(b"form-data") {
										collector_state.finished = Some(Err(CollectError::TBD));
										return;
									}
								}
							}
							println!("[mh] form-data atom found");
							let mut a_name = None::<Rdx>;
							let mut a_filename = None::<Rdx>;
							match hp.next_attribute() {
								None | Some(Err(())) => {
									collector_state.finished = Some(Err(CollectError::TBD));
									return;
								}
								Some(Ok(a)) => {
									if a.key.get(line_bytes)
										.eq_ignore_ascii_case(b"name") {
										a_name = Some(a.value.with_base(line.from()));
									} else if a.key.get(line_bytes)
										.eq_ignore_ascii_case(b"filename") {
										a_filename = Some(a.value.with_base(line.from()));
									}
								}
							}
							println!("first attribute, a_name: {:?}, a_filename: {:?}",
									 a_name, a_filename);
							if let Some(n) = a_name {
								println!("a_name: {:?}",
										 String::from_utf8_lossy(n.get(buffer)));
							}

							match hp.next_attribute() {
								None => {
									if a_name.is_none() {
										collector_state.finished = Some(Err(CollectError::TBD));
										return;
									}
								}
								Some(Err(())) => {
									collector_state.finished = Some(Err(CollectError::TBD));
									return;
								}
								Some(Ok(a)) => {
									if a.key.get(line_bytes)
										.eq_ignore_ascii_case(b"name") {
										if a_name.is_some() {
											collector_state.finished = Some(Err(CollectError::TBD));
											return;
										}
										a_name = Some(a.value.with_base(line.from()));
									} else if a.key.get(line_bytes)
										.eq_ignore_ascii_case(b"filename") {
										if a_filename.is_some() {
											collector_state.finished = Some(Err(CollectError::TBD));
											return;
										}
										a_filename = Some(a.value.with_base(line.from()));
									}
								}
							}
							println!("second attribute, a_name: {:?}, a_filename: {:?}",
									 a_name, a_filename);
							if let Some(n) = a_name {
								println!("a_name: {:?}",
										 String::from_utf8_lossy(n.get(buffer)));
							}

							self.current_attachment.name = a_name.unwrap();
							self.current_attachment.filename = a_filename;
						} else if f_name.eq_ignore_ascii_case(b"content-type") {} else {
							collector_state.finished = Some(Err(CollectError::TBD));
							return;
						}
					}
					ProcMultipart::Data => {
						println!("multipart data");

						let boundary = self.boundary_rdx.unwrap().get(buffer);
						match collector_state.state_reader.take_attachment(buffer, boundary) {
							Poll::Pending => return,
							Poll::Ready(bi) => {
								collector_state.stage =
									ProcStage::Body(
										ProcBody::Multipart(
											ProcMultipart::Headers));

								self.attachments.last_mut().unwrap().data = bi.data;

								println!("attachment completed, data:\n{:.100?}",
								String::from_utf8_lossy(bi.data.get(buffer)));

								if bi.is_last {
									collector_state.finished = Some(Ok(()));
									return;
								}
							}
						}
					}
				}
			}
		}
	}

	fn process_header_line(
		&mut self,
		buffer: &[u8],
		line: Rdx,
		collector_state: &mut CollectorState,
	) {
		let line_bytes = line.get(buffer);
		if line_bytes.trim_ascii().is_empty() {
			if !line_bytes.is_empty() {
				collector_state.finished = Some(Err(CollectError::TBD));
				return;
			}
			collector_state.stage = ProcStage::PreBody;
			return;
		}

		let mut hp = match HeaderParser::new(line_bytes) {
			Err(()) => {
				collector_state.finished = Some(Err(CollectError::InvalidHeader));
				return;
			}
			Ok(hp) => hp
		};

		{
			let h = TmpHeader {
				name: hp.field_name().with_base(line.from()),
				value: hp.field_body().with_base(line.from()),
			};
			println!(
				"pushed header: {:?}:{:?}",
				String::from_utf8_lossy(h.name.get(buffer)),
				String::from_utf8_lossy(h.value.get(buffer)),
			);
			self.headers.push(h);
		}

		let f_name = hp.field_name()
			.get(line_bytes);

		if f_name.eq_ignore_ascii_case(b"content-length") {
			let value = hp.field_body()
				.get(line_bytes);
			match su8_to_dec(value) {
				Err(()) => {
					collector_state.finished = Some(Err(CollectError::InvalidHeader));
					return;
				}
				Ok(v) => {
					if self.content_length.is_some() {
						collector_state.finished = Some(Err(CollectError::TBD));
						return;
					}
					self.content_length = Some(v);
				}
			}
		} else if f_name.eq_ignore_ascii_case(b"content-type") {
			match hp.next_type() {
				Err(()) => {
					collector_state.finished = Some(Err(CollectError::InvalidContentTypeHeader));
					return;
				}
				Ok(ct) => {
					let ct_main = ct.main.get(line_bytes);
					let ct_sub = ct.sub.get(line_bytes);
					if ct_main.eq_ignore_ascii_case(b"multipart") &&
						ct_sub.eq_ignore_ascii_case(b"form-data") {
						match hp.next_attribute() {
							None | Some(Err(())) => {
								collector_state.finished =
									Some(Err(CollectError::InvalidContentTypeHeader));
								return;
							}
							Some(Ok(at)) => {
								if !at.key.get(line_bytes)
									.eq_ignore_ascii_case(
										b"boundary") {
									collector_state.finished =
										Some(Err(CollectError::InvalidContentTypeHeader));
									return;
								}

								self.boundary_rdx = Some(at.value.with_base(line.from()));
							}
						}
					}
				}
			}
		}
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
