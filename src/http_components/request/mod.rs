use std::fmt::{Debug, Display};
use std::io::Write;
use crate::{HTTPHeader, Url, MimeType, HTTPAttachment};
use crate::http_components::message::HTTPMessage;
use crate::MimeType::Multipart;


#[derive(Clone)]
pub struct HTTPRequest {
	pub method: String,
	pub url: Url,
	pub message: HTTPMessage,
}

impl Default for HTTPRequest {
	fn default() -> Self {
		Self {
			method: String::from("GET"),
			url: Url::default(),
			message: HTTPMessage::default(),
		}
	}
}

impl HTTPRequest {
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut ret = Vec::<u8>::new();

		write!(
			&mut ret,
			"{} {} {}\r\n",
			self.method,
			self.url.get_request_target(),
			self.message.http_version
		)
			.expect("writing bytes to vec");

		ret.append(
			&mut self.message.to_bytes()
		);

		ret
	}

	pub fn get_attachments(&self) -> Option<Vec<HTTPAttachment>> {
		todo!()
	}
}

impl Debug for HTTPRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "HTTP request (version={})", self.http_version)?;
		writeln!(f, "| method={}", self.method)?;
		writeln!(f, "| path={}", self.url.path)?;
		writeln!(f, "| query={:?}", self.url.query_string)?;
		writeln!(f, "| headers:")?;
		for h in &self.headers {
			writeln!(f, "| - [{}]: [{}]", h.name, h.value)?;
		}
		writeln!(f, "| mime-type={:?}", self.mime_type)?;

		if self.mime_type == Multipart {
			writeln!(f, "| body - multipart")?;
			writeln!(f, "| attachments:")?;
			for attachment in &self.attachments {
				writeln!(f, "{:?}", attachment)?;
			}
		} else {
			writeln!(f, "| body, length={}:", self.body.len())?;
			if self.body.len() < 0x1000 {
				writeln!(f, "| <<{}>>", String::from_utf8_lossy(self.body.as_slice()))?;
			} else {
				writeln!(f, "| [body too long to display]")?;
			}
		}
		writeln!(f, "| that's all.")?;
		Ok(())
	}
}

impl Display for HTTPRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", String::from_utf8_lossy(self.to_bytes().as_slice()))
	}
}
