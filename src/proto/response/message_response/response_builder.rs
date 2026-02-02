use crate::consts::{StatusCode, Version};
use crate::proto::message::MessageBuilder;
use crate::response::Response;

pub struct ResponseBuilder {
	status_code: StatusCode,
	version: Version,
	message_builder: MessageBuilder,
}

impl Default for ResponseBuilder {
	fn default() -> Self {
		Self {
			status_code: StatusCode::SUCCESS,
			version: Version::HTTP11,
			message_builder: Default::default(),
		}
	}
}

impl ResponseBuilder {
	pub fn new() -> Self{
		Self {
			status_code: StatusCode::SUCCESS,
			version: Version::HTTP11,
			message_builder: Default::default(),
		}
	}

	pub fn into_response(self) -> Response {
		Response {
			version: self.version,
			status_code: self.status_code,
			message: self.message_builder.into_message(),
		}
	}
}

impl ResponseBuilder {
	pub fn set_status(&mut self, status: StatusCode) -> &mut Self {
		self.status_code = status;
		self
	}
	pub fn push_header(&mut self, k: &str, v: &str) -> &mut Self {
		self.message_builder.push_header(k, v);
		self
	}
}
