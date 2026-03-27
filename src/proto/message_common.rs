use crate::proto::state_buffer_reader::StateBufferReader;

pub struct MessageCommon {
	working_buffer: Vec<u8>,
	working_buffer_reader: StateBufferReader
}

impl MessageCommon {
	pub fn push_bytes(&mut self, bytes: &[u8]) {
		self.working_buffer.extend_from_slice(bytes);
	}
}
