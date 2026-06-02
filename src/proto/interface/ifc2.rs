use super::*;
use crate::proto::consts::{StatusCode, Version};
use crate::proto::message_collector::CollectError;

impl ResponseCollector {
	pub fn new() -> Self { todo!() }
	pub fn to_response(self) ->
	Result<ResponseCollectorFinished, CollectError> { todo!() }
}

impl ResponseCollectorFinished {
	pub fn status_code(&self) -> StatusCode { todo!() }
	pub fn status_description(&self) -> &[u8] { todo!() }
}

impl ResponseBuilder {
	pub fn new() -> Self { todo!() }
	pub fn as_bytes(&self) -> &[u8] { todo!() }
	pub fn quick_() -> Self { todo!() }
	pub fn set_headers(&mut self, headers: &[()]) -> &mut Self { todo!() }
}
