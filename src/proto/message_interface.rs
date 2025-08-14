use crate::proto::internal::get_message_ref_trait::GetMessageRefInternal;

use crate::{HTTPHeader, MimeType};

pub trait HTTPMessageInterface: GetMessageRefInternal {
	fn get_mime_type(&self) -> MimeType {
		self.get_message().mime_type.clone()
	}

	fn set_mime_type(&mut self, mime_type: MimeType) -> &mut Self {
		self.get_message_mut().mime_type = mime_type;
		self
	}

	fn headers(&self) -> &Vec<HTTPHeader> {
		&self.get_message().headers
	}

	fn headers_mut(&mut self) -> &mut Vec<HTTPHeader> {
		&mut self.get_message_mut().headers
	}
}
