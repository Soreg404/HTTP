use std::fmt::{Debug, Display};
use std::io::Write;
use crate::{HTTPHeader, HTTPParseError, HTTPPartialRequest};
use crate::proto::attachment::HTTPAttachment;
use crate::proto::internal::message::HTTPMessage;
use crate::proto::mime_type::MimeType::Multipart;

#[derive(Clone)]
pub struct HTTPRequest {
	pub(super) method: String,
	pub(super) target: String,
	pub(super) message: HTTPMessage,
}

impl Default for HTTPRequest {
	fn default() -> Self {
		Self {
			method: String::from("GET"),
			target: "/".to_string(),
			message: Default::default(),
		}
	}
}

impl HTTPRequest {
	pub fn set_method(&mut self, method: &str) {
		self.method = method.to_string();
	}
	pub fn get_method(&self) -> &str {
		self.method.as_str()
	}
	pub fn set_target(&mut self, target: &str) {
		self.target = target.to_string();
	}
	pub fn get_target(&self) -> &str {
		self.target.as_str()
	}
}

impl HTTPRequest {
	pub fn headers(&self) -> &Vec<HTTPHeader> {
		&self.message.headers
	}
	pub fn headers_mut(&mut self) -> &mut Vec<HTTPHeader> {
		&mut self.message.headers
	}
}

impl HTTPRequest {
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut ret = Vec::<u8>::new();

		write!(
			&mut ret,
			"{} {} HTTP/{}.{}\r\n",
			self.method,
			self.target,
			self.message.http_version.0,
			self.message.http_version.1,
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
		writeln!(f, "HTTP request (version={:?})", self.message.http_version)?;
		writeln!(f, "| method={}", self.method)?;
		writeln!(f, "| target={}", self.target)?;
		writeln!(f, "| headers:")?;
		for h in &self.message.headers {
			writeln!(f, "| - [{}]: [{}]", h.name, h.value)?;
		}
		// writeln!(f, "| mime-type={:?}", self.message.mime_type)?;

		/*match self.message.mime_type {
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
		}*/

		writeln!(f, "| that's all.")?;
		Ok(())
	}
}

impl Display for HTTPRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", String::from_utf8_lossy(self.to_bytes().as_slice()))
	}
}
