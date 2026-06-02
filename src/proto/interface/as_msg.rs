use crate::{RequestCollector, RequestCollectorFinished, ResponseCollector, ResponseCollectorFinished};
use crate::proto::consts::Version;

// as message collector
impl RequestCollector {
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize { todo!() }
	pub fn is_finished(&self) -> bool { todo!() }
}

// as message collector
impl ResponseCollector {
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize { todo!() }
	pub fn is_finished(&self) -> bool { todo!() }
}

// as message finished
impl RequestCollectorFinished {
	pub fn version(&self) -> Version { todo!() }
	pub fn headers_iter(&self) -> () { todo!() }
	pub fn body(&self) -> &[u8] { todo!() }
	pub fn attachments_iter(&self) -> &[u8] { todo!() }
}

// as message finished
impl ResponseCollectorFinished {
	pub fn version(&self) -> Version { todo!() }
	pub fn headers_iter(&self) -> () { todo!() }
	pub fn body(&self) -> &[u8] { todo!() }
	pub fn attachments_iter(&self) -> &[u8] { todo!() }
}
