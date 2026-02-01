use crate::proto::collector_states::CollectorStates;
use crate::proto::internal::buffer_reader::{BufferReader, BufferReaderTakes};

pub trait CollectorInterface: GetCollector + OnFirstLine {
	fn push_bytes(&mut self, bytes: &[u8]) -> usize {
		let collector = self.get_collector_mut();
		collector.master_buffer.push_bytes(bytes);
		collector.process(&self);
		collector.master_buffer.bytes_remaining()
	}
	fn connection_closed(&mut self) {}
}

pub trait GetCollector {
	fn get_collector(&self) -> &Collector;
	fn get_collector_mut(&mut self) -> &mut Collector;
}

pub trait OnFirstLine {
	fn on_first_line(&mut self, line: &[u8]);
}

pub struct Collector {
	master_buffer: BufferReader,
	state: CollectorStates,

	// chunk_prefix_buffer: Option<Box<[u8]>>,
	// compressed_buffer: _

	next_body_chunk_len: Option<usize>,
}

impl Collector {
	fn process(&mut self, obj: &dyn CollectorInterface) {
		use CollectorStates::*;
		match self.state {
			FirstLine
		}
	}
}
