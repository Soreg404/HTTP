mod collector;
mod builder;

pub use collector::*;
pub use builder::*;
use crate::consts::Version;

#[derive(Debug)]
pub struct Message {
	version: Version,
	headers: Vec<(String, String)>,
	body: Vec<u8>,
}

impl Message {
	pub fn version(&self) -> Version {
		self.version
	}
}

impl Message {
	pub fn into_bytes(self) -> Vec<u8> {
		let mut ret = Vec::new();

		ret.extend_from_slice(
			self.headers
				.iter()
				.map(|(k, v)| format!("{}: {}\r\n", k, v))
				.collect::<String>()
				.as_bytes()
		);

		ret.extend_from_slice(b"\r\n");

		ret.extend_from_slice(self.body.as_slice());

		ret
	}
}
