use std::fmt::{Debug, Formatter};
use crate::{HTTPHeader, MimeType};
pub struct HTTPAttachment {
	pub name: String,
	pub headers: Vec<HTTPHeader>,
	pub mime_type: MimeType,
	pub filename: Option<String>,
	pub data: Vec<u8>,
}

impl Debug for HTTPAttachment {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		writeln!(
			f,
			"[{:?}; {:?}; {:?}; len={}]",
			self.name,
			self.mime_type,
			self.filename,
			self.data.len()
		)?;

		Ok(())
	}
}
