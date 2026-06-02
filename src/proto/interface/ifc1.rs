use super::*;
use crate::proto::consts::{Method, Version};
use crate::proto::message_collector::CollectError;

impl RequestCollector {
	pub fn new() -> Self { todo!() }
	pub fn to_request(self) ->
	Result<RequestCollectorFinished, CollectError> { todo!() }
}

impl RequestCollectorFinished {
	pub fn method(&self) -> Method { todo!() }
	pub fn url(&self) -> () { todo!() }
}

impl RequestBuilder {
	pub fn new() -> Self { todo!() }
	pub fn as_bytes(&self) -> &[u8] { todo!() }
	pub fn quick_() -> Self { todo!() }
	pub fn set_headers(&mut self, headers: &[()]) -> &mut Self { todo!() }
}

