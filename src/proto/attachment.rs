use std::fmt::{Debug, Formatter};
use crate::{HTTPHeader, MimeType};

#[derive(Default, Clone)]
pub struct HTTPAttachment {
	pub headers: Vec<HTTPHeader>,
	pub mime_type: MimeType,
	pub name: String,
	pub filename: Option<String>,
	pub body: Vec<u8>,
}

impl Debug for HTTPAttachment {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		writeln!(
			f,
			"[{:?}; {:?}; {:?}; len={}]",
			self.name,
			self.mime_type,
			self.filename,
			self.body.len()
		)?;

		if self.body.len() < 100 {
			writeln!(f, "data: <<{}>>", String::from_utf8_lossy(&self.body))?;
		} else {
			writeln!(f, "[data too long to display]")?;
		}

		Ok(())
	}
}
