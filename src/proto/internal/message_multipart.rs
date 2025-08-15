use crate::HTTPHeader;

#[derive(Debug, Clone)]
pub struct HTTPMessageMultipart {
	pub http_version: (u8, u8),
	pub headers: Vec<HTTPHeader>,

}

#[cfg(feature = "bench")]
impl Default for HTTPMessageMultipart {
	fn default() -> Self {
		Self {
			http_version: (1, 1),
			headers: Vec::default(),
			body: Vec::default(),
		}
	}
}
