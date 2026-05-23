use crate::proto::consts::{Method, Version};
use crate::proto::message_collector::{CollectError, MessageCollector, MessageCollectorFinished, MessageType};
use crate::proto::state_reader::Poll;
use crate::proto::url::UrlInfo;
use std::fmt::{Debug, Formatter};

pub struct RequestCollector {
	buffer: Vec<u8>,
	message_collector: MessageCollector,
	request_basic: RequestInfo,
}

struct RequestInfo {
	method: Method,
	url: UrlInfo,
}

impl RequestCollector {
	pub fn new() -> Self {
		Self {
			buffer: vec![],
			message_collector: MessageCollector::new(MessageType::Request),
			request_basic: RequestInfo {
				method: Method::UNKNOWN,
				url: Default::default(),
			},
		}
	}

	pub fn push_bytes(&mut self, bytes: &[u8]) {
		// todo handle message too big
		self.buffer.extend_from_slice(bytes);

		match self.message_collector.first_line(&self.buffer).clone() {
			None => {}
			Some(Poll::Pending) => return,
			Some(Poll::Ready(line)) => {
				let r = match self
					.request_basic
					.parse_request_line(
						line.get(&self.buffer),
						&mut self.message_collector.version,
					) {
					Ok(()) => None,
					Err(e) => Some(Err(e.clone()))
				};
				self.message_collector.pass_first_line_parse_result(r);
			}
		};

		self.message_collector.process_buffer(&self.buffer[..]);
	}

	pub fn parse_result(&self) -> Option<Result<(), CollectError>> {
		self.message_collector.finished
	}

	pub fn finish(self) -> Option<RequestCollectorFinished> {
		println!("finish request collector");
		match self.message_collector.finished {
			None | Some(Err(_)) => None,
			Some(Ok(_)) => {
				Some(RequestCollectorFinished {
					buffer: self.buffer,
					method: self.request_basic.method,
					url: self.request_basic.url,
					msg: MessageCollectorFinished::from(self.message_collector),
				})
			}
		}
	}
}

impl RequestInfo {
	fn parse_request_line(&mut self, line_bytes: &[u8], version: &mut Version)
						  -> Result<(), CollectError> {
		let mut i = 0;
		while i < line_bytes.len() {
			if line_bytes[i] == b' ' {
				break;
			} else if !line_bytes[i].is_ascii_uppercase() {
				return Err(CollectError::IllegalCharacter)
			}
			i += 1;
		}
		if i == line_bytes.len() {
			return Err(CollectError::InvalidRequestLine);
		}
		self.method = Method::from_bytes(&line_bytes[..i]);
		i += 1;

		let url_start = i;
		while i < line_bytes.len() {
			if line_bytes[i] == b' ' {
				break;
			}
			i += 1;
		}
		if i == line_bytes.len() {
			return Err(CollectError::InvalidRequestLine);
		}
		self.url = match UrlInfo::parse_bytes(&line_bytes[url_start..i]) {
			Err(()) => return Err(CollectError::InvalidUrl),
			Ok(mut v) => {
				v.path = v.path.offset(url_start);
				v.query_string = v.query_string.map(|v| v.offset(url_start));
				v.fragment = v.fragment.map(|v| v.offset(url_start));
				v
			}
		};
		i += 1;

		*version = match Version::from_bytes(&line_bytes[i..]) {
			Err(()) => return Err(CollectError::InvalidVersion),
			Ok(v) => v
		};

		Ok(())
	}
}

pub struct RequestCollectorFinished {
	buffer: Vec<u8>,
	method: Method,
	url: UrlInfo,
	msg: MessageCollectorFinished,
}

impl Debug for RequestCollectorFinished {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Request {{")?;
		writeln!(f, "  method: {:?}", self.method)?;
		writeln!(f, "  url:")?;
		writeln!(f, "    path:     {:?}",
				 String::from_utf8_lossy(self.url.path.get(&self.buffer)))?;
		writeln!(f, "    query:    {:?}",
				 self.url.query_string.map(
					 |v| String::from_utf8_lossy(v.get(&self.buffer))))?;
		writeln!(f, "    fragment: {:?}",
				 self.url.fragment.map(
					 |v| String::from_utf8_lossy(v.get(&self.buffer))))?;
		writeln!(f, "  version: {:?}", self.msg.version)?;
		write!(f, "  headers ({})", self.msg.headers.len())?;
		if self.msg.headers.len() > 0 {
			write!(f, ":\n")?;
			for header in &self.msg.headers {
				let field_name = header.name.get(&self.buffer);
				let field_body = header.body.get(&self.buffer);

				let field_name = String::from_utf8_lossy(field_name);
				let field_body = String::from_utf8_lossy(field_body);

				writeln!(f, "    {field_name:?}: {field_body:?}")?;
			}
		} else {
			write!(f, "\n")?;
		}
		write!(f, "  body ({} bytes)", self.msg.body.len())?;
		if self.msg.body.len() > 0 {
			write!(f, ":\n")?;
			write!(f, "<<<<<{:.100}>>>>",
				   String::from_utf8_lossy(self.msg.body.get(&self.buffer)))?;
			if self.msg.body.len() > 100 {
				write!(f, "[+{} bytes]", self.msg.body.len() - 100)?;
			}
		}
		write!(f, "\n")?;

		write!(f, "  attachments ({})", self.msg.attachments.len())?;
		if self.msg.attachments.len() > 0 {
			write!(f, ":\n")?;
			for a in &self.msg.attachments {
				let name = String::from_utf8_lossy(
					a.name.unwrap().get(&self.buffer)
				);
				let filename = a.filename.map(
					|v| String::from_utf8_lossy(
						v.get(&self.buffer)
					));

				writeln!(f, "  name:     {:?}", name)?;
				writeln!(f, "  filename: {:?}", filename)?;
				writeln!(f, "  mime:     {:?}", a.mime_type)?;
				writeln!(f, "  data:")?;
				write!(f, "<<<<<{:.100?}>>>>",
					   String::from_utf8_lossy(a.data.get(&self.buffer)))?;
				if a.data.len() > 100 {
					write!(f, "[+{} bytes]", a.data.len() - 100)?;
				}
				write!(f, "\n")?;
			}
		} else {
			write!(f, "\n")?;
		}

		write!(f, "}}")?;

		Ok(())
	}
}
