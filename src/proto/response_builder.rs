use crate::proto::consts::{StatusCode, Version};

pub struct ResponseBuilder {
	pub status_code: StatusCode,
	pub status_description: String,
	pub version: Version,

	pub headers: Vec<Vec<u8>>,

	pub body: Vec<u8>,
}

impl ResponseBuilder {
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut v = Vec::<u8>::new();
		v.extend_from_slice(self.version.to_string().as_bytes());
		v.push(b' ');
		v.extend_from_slice((self.status_code as usize).to_string().as_bytes());
		v.push(b' ');
		v.extend_from_slice(self.status_description.as_bytes());
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
