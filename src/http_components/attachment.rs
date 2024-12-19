use crate::HTTPHeader;

pub struct HTTPRequestAttachment {
	pub headers: Vec<HTTPHeader>,
	pub data: Vec<u8>
}
