use crate::proto::consts::{Method, StatusCode, Version};
use crate::proto::message_collector::CollectError;

/*
 *************
 * COLLECTOR *
 *************
 */

// ↑ REQUEST COLLECTOR
struct RequestCollector;
impl RequestCollector {
	pub fn new() -> Self { todo!() }
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize { todo!() }
	pub fn is_finished(&self) -> bool { todo!() }
	pub fn to_request(self) ->
	Result<RequestCollectorFinished, CollectError> { todo!() }
}

// ↑ REQUEST COLLECTOR — FINISHED
struct RequestCollectorFinished;
impl RequestCollectorFinished {
	pub fn method(&self) -> Method { todo!() }
	pub fn url(&self) -> () { todo!() }

	pub fn version(&self) -> Version { todo!() }
	pub fn headers_iter(&self) -> () { todo!() }
	pub fn body(&self) -> &[u8] { todo!() }
	pub fn attachments_iter(&self) -> &[u8] { todo!() }
}

// ↓ RESPONSE COLLECTOR
struct ResponseCollector {}
impl ResponseCollector {
	pub fn new() -> Self { todo!() }
	pub fn push_bytes(&mut self, bytes: &[u8]) -> usize { todo!() }
	pub fn is_finished(&self) -> bool { todo!() }
	pub fn to_response(self) -> ResponseCollectorFinished { todo!() }
}

// ↓ RESPONSE COLLECTOR — FINISHED
struct ResponseCollectorFinished {}
impl ResponseCollectorFinished {
	pub fn status_code(&self) -> StatusCode { todo!() }
	pub fn status_description(&self) -> &[u8] { todo!() }

	pub fn version(&self) -> Version { todo!() }
	pub fn headers_iter(&self) -> () { todo!() }
	pub fn body(&self) -> &[u8] { todo!() }
	pub fn attachments_iter(&self) -> &[u8] { todo!() }
}

/*
 ***********
 * BUILDER *
 ***********
 */

// ↑ REQUEST BUILDER
struct RequestBuilder {}
impl RequestBuilder {
	pub fn new() -> Self { todo!() }
	pub fn as_bytes(&self) -> &[u8] { todo!() }
	pub fn quick_() -> Self { todo!() }
	pub fn set_headers(&mut self, headers: &[()]) -> &mut self { todo!() }
}

// ↑ REQUEST BUILDER MULTIPART
struct RequestBuilderMultipart {}

// ↓ RESPONSE BUILDER
struct ResponseBuilder {}
impl ResponseBuilder {
	pub fn new() -> Self { todo!() }
	pub fn as_bytes(&self) -> &[u8] { todo!() }
	pub fn quick_() -> Self { todo!() }
	pub fn set_headers(&mut self, headers: &[()]) -> &mut self { todo!() }
}

// ↓ RESPONSE BUILDER MULTIPART
struct ResponseBuilderMultipart {}
