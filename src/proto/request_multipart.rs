use crate::proto::internal::message_multipart::HTTPMessageMultipart;

#[derive(Debug, Clone)]
pub struct HTTPRequestMultipart {
	pub(super) status_code: u16,
	pub(super) status_text: String,
	pub(super) message_multipart: HTTPMessageMultipart,
}
