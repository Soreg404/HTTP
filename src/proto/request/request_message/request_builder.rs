use crate::consts::{Method, Version};
use crate::proto::message::MessageBuilder;
use crate::request::Request;

pub struct RequestBuilder {
	pub method: Method,
	pub url: String,
	pub version: Version,

	pub message_builder: MessageBuilder
}

impl RequestBuilder {
	pub fn into_request(self) -> Request {
		Request {
			method: self.method,
			url: self.url,
			message: self.message_builder.into_message(self.version),
		}
	}

	pub fn push_header(&mut self, field_name: &str, field_value: &str) -> &mut Self {
		self.message_builder.push_header(field_name, field_value);
		self
	}
}
