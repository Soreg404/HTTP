pub use crate::proto::message_collector::CollectError;

pub use crate::proto::message_collector::request_collector::RequestCollector;

pub struct RequestCollectorFinished;

pub struct RequestBuilder;

pub struct ResponseCollector;
pub struct ResponseCollectorFinished;
pub struct ResponseBuilder;


impl RequestCollectorFinished {
	pub fn headers(&self) -> ! {
		// self.message.headers();
		todo!()
	}
}
