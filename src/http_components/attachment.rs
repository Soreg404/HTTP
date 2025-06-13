use crate::{HTTPHeader, MimeType};
pub struct HTTPAttachment {
	number: usize,
	mime: MimeType,
	headers: Vec<HTTPHeader>,
	data: Vec<u8>,
}
