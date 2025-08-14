use std::fmt::{Debug, Display};
use std::io::Write;
use crate::HTTPMessageInterface;
use crate::proto::attachment::HTTPAttachment;
use crate::proto::internal::get_message_ref_trait::GetMessageRefInternal;
use crate::proto::internal::message::HTTPMessage;
use crate::proto::mime_type::MimeType::Multipart;
use crate::proto::Url;

#[derive(Clone)]
pub struct HTTPRequest {
	pub(super) method: String,
	pub(super) url: Url,
	pub(super) message: HTTPMessage,
}

impl GetMessageRefInternal for HTTPRequest {
	fn get_message(&self) -> &HTTPMessage { &self.message }
	fn get_message_mut(&mut self) -> &mut HTTPMessage { &mut self.message }
}

impl HTTPMessageInterface for HTTPRequest {}

impl Default for HTTPRequest {
	fn default() -> Self {
		Self {
			method: String::from("GET"),
			url: Default::default(),
			message: Default::default(),
		}
	}
}

impl HTTPRequest {

	pub fn new(method: &str, url: Url) -> Self {
		Self {
			method: method.to_string(),
			url,
			message: Default::default(),
		}
	}

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
		writeln!(f, "HTTP request (version={})", self.message.http_version)?;
		writeln!(f, "| method={}", self.method)?;
		writeln!(f, "| path={}", self.url.path)?;
		writeln!(f, "| query={:?}", self.url.query_string)?;
		writeln!(f, "| headers:")?;
		for h in &self.message.headers {
			writeln!(f, "| - [{}]: [{}]", h.name, h.value)?;
		}
		writeln!(f, "| mime-type={:?}", self.message.mime_type)?;

		match self.message.mime_type {
			Multipart(_) => {
				writeln!(f, "| body - multipart")?;
				writeln!(f, "| attachments:")?;
				for attachment in &self.message.attachments {
					writeln!(f, "{:?}", attachment)?;
				}
			}
			_ => {
				writeln!(f, "| body, length={}:", self.message.body.len())?;
				if self.message.body.len() < 0x1000 {
					writeln!(f, "| <<{}>>", String::from_utf8_lossy(self.message.body.as_slice()))?;
				} else {
					writeln!(f, "| [body too long to display]")?;
				}
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
