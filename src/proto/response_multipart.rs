use crate::{HTTPParseError, HTTPResponse};
use crate::proto::internal::message_multipart::HTTPMessageMultipart;

#[derive(Debug, Clone)]
pub struct HTTPResponseMultipart {
	pub(super) status_code: u16,
	pub(super) status_text: String,
	pub(super) message_multipart: HTTPMessageMultipart,
}

impl TryFrom<HTTPResponse> for HTTPResponseMultipart {
	type Error = HTTPParseError;
	fn try_from(response: HTTPResponse) -> Result<Self, Self::Error> {
		Ok(HTTPResponseMultipart {
			status_code: response.status_code,
			status_text: response.status_text,
			message_multipart: response.message.try_into()?,
		})
	}
}
