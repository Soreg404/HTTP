use crate::{HTTPAttachment, HTTPParseError, HTTPRequest, HTTPResponse, HTTPResponseMultipart};
use crate::proto::internal::message_multipart::HTTPMessageMultipart;

#[derive(Debug, Clone)]
pub struct HTTPRequestMultipart {
	pub(super) method: String,
	pub(super) target: String,
	pub(super) message_multipart: HTTPMessageMultipart,
}


impl HTTPRequestMultipart {
	pub fn attachments(&self) -> &[HTTPAttachment] {
		self.message_multipart.attachments.as_slice()
	}
	pub fn attachments_mut(&mut self) -> &mut Vec<HTTPAttachment> {
		&mut self.message_multipart.attachments
	}
}

impl TryFrom<HTTPRequest> for HTTPRequestMultipart {
	type Error = HTTPParseError;
	fn try_from(request: HTTPRequest) -> Result<Self, Self::Error> {
		Ok(HTTPRequestMultipart {
			method: request.method,
			target: request.target,
			message_multipart: request.message.try_into()?,
		})
	}
}
