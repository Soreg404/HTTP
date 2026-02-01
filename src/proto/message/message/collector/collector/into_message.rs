use super::MessageCollector;
use crate::proto::message::Message;

impl MessageCollector {
	pub fn into_message(self) -> Message {
		Message {
			headers: self.collected_headers
				.into_iter()
				.map(|(hfn, hfv)| (
					String::from_utf8_lossy(hfn.as_slice()).to_string(),
					String::from_utf8_lossy(hfv.as_slice()).to_string(),
				))
				.collect(),
			body: self.collected_body,
		}
	}
}
