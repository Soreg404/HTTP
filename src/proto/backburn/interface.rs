use crate::proto::message::*;

mod ifc1;
mod ifc2;
mod as_msg;

// collector
pub struct RequestCollector {
	msg: MessageCollector<RequestCollectorSpecific>
}
pub struct RequestCollectorFinished;

pub struct ResponseCollector;
pub struct ResponseCollectorFinished;

// builder
pub struct RequestBuilder;
pub struct RequestBuilderMultipart;

pub struct ResponseBuilder;
pub struct ResponseBuilderMultipart;
