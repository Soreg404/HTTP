use crate::proto::consts::Method;
use crate::proto::message::Message;

mod request_collector;
mod request_builder;

pub use request_collector::RequestCollector as Collector;
pub use request_builder::RequestBuilder as Builder;
use crate::consts::Version;

#[derive(Debug)]
pub struct MessageRequest {
	method: Method,
	url: String,
	version: Version,
	message: Message,
}
