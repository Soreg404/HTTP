use crate::proto::consts::Method;
use crate::proto::message::Message;

mod request_collector;
mod request_builder;

pub use request_collector::RequestCollector as Collector;
pub use request_builder::RequestBuilder as Builder;

#[derive(Debug)]
pub struct MessageRequest {
	/* todo: pub is temporary */
	pub method: Method,
	pub url: String,
	pub message: Message,
}
