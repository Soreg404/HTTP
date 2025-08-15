use std::convert::Infallible;
use crate::proto::header::HTTPHeader;
use crate::proto::mime_type::MimeType;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use crate::{HTTPAsciiStr, HTTPParseError};
use crate::proto::internal::parser_header::ContentDisposition;

#[derive(Default)]
pub struct HTTPAttachment {
	pub headers: Vec<HTTPHeader>,
	pub body: Vec<u8>,
}

impl HTTPAttachment {
	pub fn get_content_disposition(&self) -> Option<Result<ContentDisposition, HTTPParseError>> {
		let header = self.headers.iter().find(|h|
			h.name.eq_ignore_ascii_case("content-disposition"))?;

		// let s = match HTTPAsciiStr::try_from(header.value.as_str()) {
		// 	Ok(s) => s,
		// 	Err(e) => return Some(Err(e)),
		// };

		Some(ContentDisposition::try_from(header.value.as_str()))
	}
}

impl Debug for HTTPAttachment {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		// writeln!(
		// 	f,
		// 	"[{:?}; {:?}; {:?}; len={}]",
		// 	self.name,
		// 	self.mime_type,
		// 	self.filename,
		// 	self.body.len()
		// )?;

		if self.body.len() < 100 {
			writeln!(f, "data: <<{}>>", String::from_utf8_lossy(&self.body))?;
		} else {
			writeln!(f, "[data too long to display]")?;
		}

		Ok(())
	}
}
