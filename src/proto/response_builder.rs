use crate::proto::consts::{StatusCode, Version};

pub struct ResponseBuilder {
	pub status_code: StatusCode,
	pub status_description: Option<Vec<u8>>,
	pub version: Version,

	pub headers: Vec<Vec<u8>>,

	pub body: Vec<u8>,
}

impl ResponseBuilder {
	pub fn new() -> Self {
		Self {
			status_code: StatusCode::SUCCESS,
			status_description: None,
			version: Version::HTTP_1_1,
			headers: vec![],
			body: vec![],
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
}

impl ResponseBuilder {
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
