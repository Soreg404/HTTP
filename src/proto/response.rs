use std::fmt::{Debug, Display};
use std::io::Write;
use crate::HTTPRequest;
use crate::proto::header::HTTPHeader;
use crate::proto::internal::message::HTTPMessage;
use crate::proto::mime_type::MimeType;

#[derive(Debug, Clone)]
pub struct HTTPResponse {
	pub(super) status_code: u16,
	pub(super) status_text: String,
	pub(super) message: HTTPMessage,
}

// todo: delete default as it has to set the status_code manually
impl Default for HTTPResponse {
	fn default() -> Self {
		Self {
			status_code: 200,
			status_text: Self::status_code_to_str(200).into(),
			message: Default::default(),
		}
	}
}

impl HTTPResponse {
	pub fn set_status_code(&mut self, status_code: u16) {
		self.status_code = status_code;
	}
	pub fn get_status_code(&self) -> u16 {
		self.status_code
	}
	pub fn set_status_text(&mut self, status_text: &str) {
		self.status_text = status_text.to_string();
	}
	pub fn get_status_text(&self) -> &str {
		self.status_text.as_str()
	}
}

impl HTTPResponse {
	pub fn headers(&self) -> &Vec<HTTPHeader> {
		&self.message.headers
	}
	pub fn headers_mut(&mut self) -> &mut Vec<HTTPHeader> {
		&mut self.message.headers
	}
}

impl HTTPResponse {
	pub fn status_code_to_str(status_code: u16) -> &'static str {
		match status_code {
			200 => "OK",
			404 => "NOT FOUND",
			418 => "I'M A TEAPOT",
			500 => "INTERNAL SERVER ERROR",
			_ => "OTHER"
		}
	}
	pub fn quick(status_code: u16) -> Self {
		Self {
			status_code,
			status_text: Self::status_code_to_str(status_code).into(),

			..Default::default()
		}
	}
	pub fn new_json(json_string: &str) -> Self {
		HTTPResponse {
			status_code: 200,
			status_text: Self::status_code_to_str(200).into(),
			message: HTTPMessage {
				headers: vec![
					HTTPHeader::new(
						"content-type",
						"application/json)"
					)
				],
				body: json_string.as_bytes().to_vec(),
				..Default::default()
			},
		}
	}
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut ret = Vec::<u8>::new();

		write!(
			&mut ret,
			"HTTP/{}.{} {} {}\r\n",
			self.message.http_version.0,
			self.message.http_version.1,
			self.status_code,
			self.status_text
		)
			.expect("writing bytes to vec");

		ret.append(
			&mut self.message.to_bytes()
		);

		ret
	}
}

#[cfg(feature="bench")]
impl Debug for HTTPResponse {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "HTTP response (version={})", self.http_version)?;
		writeln!(f, "| status={} (\"{}\")", self.status, Self::status_code_str(self.status))?;
		writeln!(f, "| headers:")?;
		for h in &self.headers {
			writeln!(f, "| - [{}]: [{}]", h.name, h.value)?;
		}
		writeln!(f, "| body, length={}:", self.body.len())?;
		if self.body.len() < 0x1000 {
			writeln!(f, "| {:?}", String::from_utf8_lossy(self.body.as_slice()))?;
		} else {
			writeln!(f, "| [body too long to display]")?;
		}
		writeln!(f, "| that's all.")?;
		Ok(())
	}
}

impl Display for HTTPResponse {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		writeln!(f, "{}", String::from_utf8_lossy(self.to_bytes().as_slice()))
	}
}
