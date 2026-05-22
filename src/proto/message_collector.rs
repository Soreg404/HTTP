use crate::proto::consts::{MimeType, Version};
use crate::proto::header_parser::{header_from_line, HeaderBodyParser, HeaderRdx};
use crate::proto::rdx::Rdx;
use crate::proto::state_reader::{Poll, StateReader};

pub mod request_collector;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CollectError {
	TBD,
	IllegalCharacter,
	UnknownMethod,
	InvalidUrl,
	InvalidVersion,
	InvalidContentTypeHeader,
	InvalidHeader,
	InvalidRequestLine,
}

#[derive(Clone, Copy, Debug)]
pub enum MessageType {
	Request,
	Response,
}

#[derive(Debug, Copy, Clone)]
enum ProcStage {
	FirstLine(MessageType),
	MainHeaders,

	PreBody,
	Body(ProcBody),
}

#[derive(Debug, Clone, Copy)]
enum ProcBody {
	Normal {
		content_length: usize,
	},
	Chunked,
	Multipart {
		content_length: usize,
		boundary: Rdx,
		stage: ProcMultipart,
	},
}

#[derive(Clone, Copy, Debug)]
enum ProcMultipart {
	Init,
	Headers,
	Data,
}

pub struct MessageCollector {
	attachments: Vec<Attachment>,
	headers: Vec<HeaderRdx>,

	finished: Option<Result<(), CollectError>>,
	stage: ProcStage,
	state_reader: StateReader,

	version: Version,

	content_length: Option<usize>,
	boundary: Option<Rdx>,

	body: Option<Rdx>,
}

#[derive(Debug, Default, Clone)]
struct Attachment {
	name: Option<Rdx>,
	filename: Option<Rdx>,
	mime_type: MimeType,
	data: Rdx,
}

impl MessageCollector {
	pub fn new(message_type: MessageType) -> Self {
		Self {
			attachments: vec![],
			headers: vec![],
			finished: None,
			stage: ProcStage::FirstLine(message_type),
			state_reader: Default::default(),
			version: Version::HTTP_1_1,
			content_length: None,
			boundary: None,
			body: None,
		}
	}

	pub fn first_line(&mut self, buffer: &[u8]) -> Option<Poll<Rdx>> {
		match self.stage {
			ProcStage::FirstLine(_) => {
				Some(self.state_reader.take_line(buffer))
			}
			_ => None
		}
	}

	pub fn pass_first_line_parse_result(
		&mut self,
		finished: Option<Result<(), CollectError>>,
	) {
		match self.stage {
			ProcStage::FirstLine(_) => {
				self.finished = finished;
				self.stage = ProcStage::MainHeaders;
			}
			_ => panic!()
		}
	}

	pub fn process_buffer(
		&mut self,
		buffer: &[u8],
	) {
		while self.finished.is_none() {
			match self.stage.clone() {
				ProcStage::FirstLine(_) => unreachable!(),
				ProcStage::MainHeaders => {
					match self.state_reader.take_line(buffer) {
						Poll::Pending => return,
						Poll::Ready(line) =>
							self.process_header_line(
								buffer,
								line,
							),
					}
				}
				ProcStage::PreBody => {
					println!("pre-body");
					match self.content_length {
						None => {
							self.finished = Some(Ok(()));
							return;
						}
						Some(content_length) => {
							match self.boundary {
								None => {
									self.stage =
										ProcStage::Body(ProcBody::Normal { content_length });
								}
								Some(boundary) => {
									self.stage =
										ProcStage::Body(
											ProcBody::Multipart {
												content_length,
												boundary,
												stage: ProcMultipart::Init,
											})
								}
							}
						}
					}
				}
				ProcStage::Body(ProcBody::Normal { content_length }) => {
					println!("body normal, length: {content_length}");

					let body_start = self.state_reader.base;
					let body_end = body_start + content_length;

					if body_end > buffer.len() {
						return;
					}

					self.body = Some(Rdx::new(body_start, body_end));
					self.finished = Some(Ok(()));

					return;
				}
				ProcStage::Body(ProcBody::Chunked) => todo!(),
				ProcStage::Body(
					ProcBody::Multipart {
						content_length,
						boundary,
						stage
					}) => match stage {
					ProcMultipart::Init => {
						println!("body multipart, init, length: {content_length}, boundary: {:?}",
								 String::from_utf8_lossy(boundary.get(buffer)));

						let boundary_bytes = boundary.get(buffer);

						match self.state_reader.take_attachment(buffer, boundary_bytes) {
							Poll::Pending => return,
							Poll::Ready(bi) => {
								println!("init found attachment, data: {:?}",
										 String::from_utf8_lossy(bi.data.get(buffer)));

								if bi.is_last {
									self.finished = Some(Ok(()));
									return;
								}
								self.attachments.push(Attachment::default());
								self.stage =
									ProcStage::Body(
										ProcBody::Multipart {
											content_length,
											boundary,
											stage: ProcMultipart::Headers,
										});
							}
						};
					}
					ProcMultipart::Headers => {
						println!("multipart headers");

						let line = match self
							.state_reader.take_line(buffer) {
							Poll::Pending => return,
							Poll::Ready(line) => line
						};
						let line_bytes = line.get(buffer);

						println!("[multipart headers] line: {:?}",
								 String::from_utf8_lossy(line_bytes));

						if line_bytes.trim_ascii().is_empty() {
							if !line_bytes.is_empty() {
								self.finished = Some(Err(CollectError::TBD));
								return;
							}

							if self.attachments.last().unwrap().name.is_none() {
								self.finished = Some(Err(CollectError::TBD));
								return;
							}

							self.stage =
								ProcStage::Body(
									ProcBody::Multipart {
										content_length,
										boundary,
										stage: ProcMultipart::Data,
									});

							println!("[multipart headers] empty line");

							continue;
						}

						let header = match header_from_line(line_bytes) {
							Err(()) => {
								self.finished = Some(Err(CollectError::InvalidHeader));
								return;
							}
							Ok(v) => v
						};

						let f_name_bytes = header.name
							.get(line_bytes);
						let f_body_bytes = header.body.get(line_bytes);

						let mut hbp = HeaderBodyParser::new(
							f_body_bytes
						);

						if f_name_bytes.eq_ignore_ascii_case(b"content-disposition") {
							println!("[multipart headers] parsing content-disposition");
							match hbp.next_atom() {
								None => {
									self.finished = Some(Err(CollectError::InvalidHeader));
									return;
								}
								Some(s) => {
									if !s.get(f_body_bytes)
										.eq_ignore_ascii_case(b"form-data") {
										self.finished = Some(Err(CollectError::TBD));
										return;
									}
								}
							}
							println!("[multipart headers] form-data atom found");
							let mut a_name = None::<Rdx>;
							let mut a_filename = None::<Rdx>;
							match hbp.next_attribute() {
								None | Some(Err(())) => {
									println!("[multipart headers] first attrib error");
									self.finished = Some(Err(CollectError::InvalidHeader));
									return;
								}
								Some(Ok(a)) => {
									if a.key.get(f_body_bytes)
										.eq_ignore_ascii_case(b"name") {
										a_name = Some(a.value.with_base(
											line.from() + header.body.from()));
									} else if a.key.get(f_body_bytes)
										.eq_ignore_ascii_case(b"filename") {
										a_filename = Some(a.value.with_base(
											line.from() + header.body.from()));
									} else {
										self.finished = Some(Err(CollectError::InvalidHeader));
										return;
									}
								}
							}
							println!(
								"[multipart headers] second attribute, \
								a_name: {:?}, a_filename: {:?}",
								a_name.map(|v| String::from_utf8_lossy(v.get(buffer))),
								a_filename.map(|v| String::from_utf8_lossy(v.get(buffer)))
							);

							match hbp.next_attribute() {
								None => {
									if a_name.is_none() {
										self.finished = Some(Err(CollectError::TBD));
										return;
									}
								}
								Some(Err(())) => {
									self.finished = Some(Err(CollectError::TBD));
									return;
								}
								Some(Ok(a)) => {
									if a.key.get(f_body_bytes)
										.eq_ignore_ascii_case(b"name") {
										if a_name.is_some() {
											self.finished =
												Some(Err(CollectError::InvalidHeader));
											return;
										}
										a_name = Some(a.value.with_base(
											line.from() + header.body.from()));
									} else if a.key.get(f_body_bytes)
										.eq_ignore_ascii_case(b"filename") {
										if a_filename.is_some() {
											self.finished =
												Some(Err(CollectError::InvalidHeader));
											return;
										}
										a_filename = Some(a.value.with_base(
											line.from() + header.body.from()));
									} else {
										self.finished = Some(Err(CollectError::InvalidHeader));
										return;
									}
								}
							}
							println!(
								"[multipart headers] second attribute, \
								a_name: {:?}, a_filename: {:?}",
								a_name.map(|v| String::from_utf8_lossy(v.get(buffer))),
								a_filename.map(|v| String::from_utf8_lossy(v.get(buffer)))
							);

							self.attachments.last_mut().unwrap().name = a_name;
							self.attachments.last_mut().unwrap().filename = a_filename;
						} else if f_name_bytes.eq_ignore_ascii_case(b"content-type") {} else {
							self.finished = Some(Err(CollectError::TBD));
							return;
						}
					}
					ProcMultipart::Data => {
						println!("multipart data");

						let boundary_bytes = boundary.get(buffer);
						match self.state_reader.take_attachment(buffer, boundary_bytes) {
							Poll::Pending => return,
							Poll::Ready(bi) => {
								self.stage =
									ProcStage::Body(
										ProcBody::Multipart {
											content_length,
											boundary,
											stage: ProcMultipart::Headers,
										});

								self.attachments.last_mut().unwrap().data = bi.data;

								println!("[multipart data] attachment completed, data:\n{:.100?}",
										 String::from_utf8_lossy(bi.data.get(buffer)));

								if bi.is_last {
									self.finished = Some(Ok(()));
								} else {
									self.attachments.push(Attachment::default());
								}
							}
						}
					}
				},
			}
		}
	}

	fn process_header_line(
		&mut self,
		buffer: &[u8],
		line: Rdx,
	) {
		println!("process header line");

		let line_bytes = line.get(buffer);
		if line_bytes.trim_ascii().is_empty() {
			if !line_bytes.is_empty() {
				self.finished = Some(Err(CollectError::TBD));
				return;
			}
			println!("valid empty line");
			self.stage = ProcStage::PreBody;
			return;
		}

		let header = match header_from_line(line_bytes) {
			Err(()) => {
				self.finished = Some(Err(CollectError::InvalidHeader));
				return;
			}
			Ok(v) => v
		};

		println!(
			"pushed header: {:?}:{:?}",
			String::from_utf8_lossy(header.name.get(line_bytes)),
			String::from_utf8_lossy(header.body.get(line_bytes)),
		);
		self.headers.push(HeaderRdx {
			name: header.name.with_base(line.from()),
			body: header.body.with_base(line.from()),
		});

		let f_name = header.name.get(line_bytes);
		let f_body = header.body.get(line_bytes);
		let mut hbp = HeaderBodyParser::new(f_body);

		if f_name.eq_ignore_ascii_case(b"content-length") {
			match su8_to_dec(f_body) {
				Err(()) => {
					self.finished = Some(Err(CollectError::InvalidHeader));
					return;
				}
				Ok(v) => {
					if self.content_length.is_some() {
						self.finished = Some(Err(CollectError::TBD));
						return;
					}
					println!("found content-length header: {v}");
					self.content_length = Some(v);
				}
			}
		} else if f_name.eq_ignore_ascii_case(b"content-type") {
			match hbp.next_type() {
				Err(()) => {
					self.finished = Some(Err(CollectError::InvalidContentTypeHeader));
					return;
				}
				Ok(ct) => {
					let ct_main = ct.main.get(f_body);
					let ct_sub = ct.sub.get(f_body);
					if ct_main.eq_ignore_ascii_case(b"multipart") &&
						ct_sub.eq_ignore_ascii_case(b"form-data") {
						match hbp.next_attribute() {
							None | Some(Err(())) => {
								self.finished =
									Some(Err(CollectError::InvalidContentTypeHeader));
								return;
							}
							Some(Ok(at)) => {
								if !at.key.get(f_body)
									.eq_ignore_ascii_case(
										b"boundary") {
									self.finished =
										Some(Err(CollectError::InvalidContentTypeHeader));
									return;
								}

								self.boundary = Some(
									at.value.with_base(
										line.from() + header.body.from()
									)
								);
							}
						}
					}
				}
			}
		}
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
