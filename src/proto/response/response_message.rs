use crate::proto::consts::StatusCode;
use crate::proto::message::Message;

mod response_collector;
mod response_builder;

use crate::consts::Version;
pub use response_builder::ResponseBuilder as Builder;
pub use response_collector::ResponseCollector as Collector;

#[derive(Debug)]
pub struct MessageResponse {
	status_code: StatusCode,
	status_desc: String,
	message: Message,
}

impl MessageResponse {
	pub fn into_bytes(self) -> Vec<u8> {
		let mut ret = Vec::new();

		let first_line = format!(
			"{} {} {}\r\n",
			self.message.version(),
			self.status_code as usize,
			self.status_code.as_desc()
		);

		ret.extend_from_slice(first_line.as_bytes());

		ret.extend_from_slice(self.message.as_bytes().as_slice());

		ret
	}
}

impl MessageResponse {
	pub fn status_code(&self) -> StatusCode {
		self.status_code
	}

	pub fn body_as_bytes(&self) -> &[u8] {
		self.message.body_as_bytes()
	}
}
