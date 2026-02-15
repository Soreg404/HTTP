use crate::proto::consts::Method;
use crate::proto::message::Message;

mod request_collector;
mod request_builder;

pub use request_collector::RequestCollector as Collector;
pub use request_builder::RequestBuilder as Builder;

#[derive(Debug)]
pub struct MessageRequest {
	/* todo: pub is temporary */
	pub method: Method,
	pub url: String,
	pub message: Message,
}

impl MessageRequest {
	pub fn as_bytes(&self) -> Vec<u8> {

		let mut ret = Vec::new();

		let first_line = format!(
			"{} {} {}\r\n",
			self.method,
			self.url,
			self.message.version()
		);

		ret.extend_from_slice(first_line.as_bytes());

		ret.extend_from_slice(self.message.as_bytes().as_slice());

		ret
	}
}
