use crate::proto::message::Message;

pub struct MessageBuilder {
	headers: Vec<(String, String)>,
	body: Vec<u8>,
}

impl Default for MessageBuilder {
	fn default() -> Self {
		Self {
			headers: vec![],
			body: vec![],
		}
	}
}

impl MessageBuilder {
	pub fn push_header(&mut self, field_name: &str, field_value: &str)
	-> &mut Self {
		self.headers.push((field_name.to_string(), field_value.to_string()));
		self
	}
}

impl MessageBuilder {
	pub fn into_message(self) -> Message {
		Message {
			headers: self.headers,
			body: self.body,
		}
	}
}
