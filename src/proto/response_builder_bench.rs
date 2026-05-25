use crate::Method;
use crate::proto::consts::{StatusCode, Version};

pub struct ResponseBuilder {
	intl: ResponseBuilderInternal,
}

pub struct ResponseBuilderMultipart {
	intl: ResponseBuilderInternal,
}

struct ResponseBuilderInternal {
	status_code: StatusCode,
	status_description: Option<Vec<u8>>,
	msg: MessageBuilder
}

pub struct RequestBuilder {
	method: Method,
	url: Vec<u8>,
	msg: MessageBuilder
}

struct MessageBuilder {
	version: Version,

	headers: Vec<Vec<u8>>,

	body: Vec<u8>,
}

trait ResponseBuilderInterfaceIntermediate {
	fn intl(&mut self) -> &mut ResponseBuilderInternal;
}

impl ResponseBuilderInterfaceIntermediate for ResponseBuilder {
	fn intl(&mut self) -> &mut ResponseBuilderInternal {
		&mut self.intl
	}
}

impl ResponseBuilderInterfaceIntermediate for ResponseBuilderMultipart {
	fn intl(&mut self) -> &mut ResponseBuilderInternal {
		&mut self.intl
	}
}

trait MessageBuilderInterfaceIntermediate {
	fn msg(&mut self) -> &mut MessageBuilder;
}

impl MessageBuilderInterfaceIntermediate for ResponseBuilderInternal {
	fn msg(&mut self) -> &mut MessageBuilder {
		&mut self.msg
	}
}

impl MessageBuilderInterfaceIntermediate for RequestBuilder {
	fn msg(&mut self) -> &mut MessageBuilder {
		&mut self.msg
	}
}

pub trait ResponseBuilderInterface : ResponseBuilderInterfaceIntermediate {
	
}

impl ResponseBuilder {
	pub fn new() -> Self {
		Self {
			intl: ResponseBuilderInternal {},
		}
	}

	pub fn new_multipart() -> ResponseBuilderMultipart {
		ResponseBuilderMultipart {

		}
	}

	pub fn new_status(code: StatusCode) -> Self {
		Self {
			status_code: code,
			status_description: None,
			version: Version::HTTP_1_1,
			headers: vec![],
			body: vec![],
		}
	}
	pub fn status(&mut self, status_code: StatusCode) -> &mut self {
		self.status_code = status_code;
		self
	}
	pub fn status_desc(&mut self, status_description: Option<Vec<u8>>) -> &mut Self {
		self.status_description = status_description;
		self
	}

	pub fn set_headers(&mut self, headers: Vec<Vec<u8>>) -> &mut Self {
		self.headers = headers;
		self
	}
	pub fn set_body(&mut self, body: Vec<u8>) -> &mut Self {
		self.body = body;
		self
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		let mut v = Vec::<u8>::new();
		v.extend_from_slice(self.version.to_string().as_bytes());
		v.push(b' ');
		v.extend_from_slice((self.status_code as usize).to_string().as_bytes());
		v.push(b' ');
		v.extend_from_slice(
			match &self.status_description {
				None => self.status_code.as_desc(),
				Some(v) => v.as_slice()
			}
		);
		v.extend_from_slice(b"\r\n");

		for header in &self.headers {
			v.extend_from_slice(&header);
			v.extend_from_slice(b"\r\n");
		}
		if !self.body.is_empty() {
			v.extend_from_slice(b"content-length: ");
			v.extend_from_slice(self.body.len().to_string().as_bytes());
			v.extend_from_slice(b"\r\n");
		}
		v.extend_from_slice(b"\r\n");

		v.extend_from_slice(self.body.as_slice());

		v
	}

	pub fn quick_404() -> Self {
		Self {
			status_code: StatusCode::NOT_FOUND,
			status_description: "NOT FOUND".to_string(),
			version: Version::HTTP_1_1,
			headers: vec![],
			body: vec![],
		}
	}
}
