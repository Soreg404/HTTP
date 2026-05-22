use crate::proto::consts::{Method, Version};
use crate::proto::message_collector::{CollectError, MessageCollector, MessageType};
use crate::proto::state_reader::Poll;
use crate::proto::url::UrlInfo;

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
						&mut self.message_collector.version,
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
			return Err(CollectError::InvalidRequestLine)
		}
		Method::from_bytes(&line_bytes[..i]);
		i += 1;
		println!("dbg method: {:?}", self.method);

		let url_start = i;
		while i < line_bytes.len() {
			if line_bytes[i] == b' ' {
				break;
			}
			i += 1;
		}
		if i == line_bytes.len() {
			return Err(CollectError::InvalidRequestLine)
		}
		println!("dbg url_bytes: {:?}", String::from_utf8_lossy(&line_bytes[url_start..i]));
		self.url = match UrlInfo::parse_bytes(&line_bytes[url_start..i]) {
			Err(()) => return Err(CollectError::InvalidUrl),
			Ok(mut v) => {
				v.path = v.path.with_base(url_start);
				v.query_string = v.query_string.map(|v| v.with_base(url_start));
				v.fragment = v.fragment.map(|v| v.with_base(url_start));

				println!("url_info:");
				println!("path: {:?}", String::from_utf8_lossy(v.path.get(line_bytes)));
				println!("query: {:?}",
						 v.query_string.map(
							 |v| String::from_utf8_lossy(v.get(line_bytes))));
				println!("frag: {:?}",
						 v.fragment.map(
							 |v| String::from_utf8_lossy(v.get(line_bytes))));

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
