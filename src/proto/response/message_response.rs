use crate::proto::consts::StatusCode;
use crate::proto::message::Message;

mod response_collector;
mod response_builder;

pub use response_collector::ResponseCollector as Collector;
pub use response_builder::ResponseBuilder as Builder;

pub struct MessageResponse {
	status_code: StatusCode,
	message: Message,
}
