use crate::StatusCode;

pub struct ResponseBuilder {
	intl_rb: ResponseBuilderInternal,
	intl_msg: MessageBuilder,
}
pub struct ResponseBuilderMultipart {
	intl_rb: ResponseBuilderInternal,
	intl_msg: MessageBuilderMultipart,
}

pub struct RequestBuilder {
	intl_rb: RequestBuilderInternal,
	intl_msg: MessageBuilder,
}
pub struct RequestBuilderMultipart {
	intl_rb: RequestBuilderInternal,
	intl_msg: MessageBuilderMultipart,
}


impl ResponseBuilder {
	pub fn new() -> ResponseBuilder {}
	pub fn new_multipart() -> RequestBuilderMultipart {}
}


//
// PRIVATE IMPLEMENTATIONS
//

struct ResponseBuilderInternal {
	status_code: StatusCode,
}
struct RequestBuilderInternal {}

struct MessageBuilder {}
struct MessageBuilderMultipart {}


// RESPONSE BUILDER INTERFACE
pub trait ResponseBuilderInterfaceIntermediate {
	fn intl(&mut self) -> &mut ResponseBuilderInternal;
}
pub trait ResponseBuilderInterface: ResponseBuilderInterfaceIntermediate {
	fn status(&mut self, status_code: StatusCode) -> &mut self {
		self.intl().status_code = status_code;
		self
	}
}

impl ResponseBuilderInterfaceIntermediate for ResponseBuilder {
	fn intl(&mut self) -> &mut ResponseBuilderInternal {
		&mut self.intl_rb
	}
}
impl ResponseBuilderInterfaceIntermediate for ResponseBuilderMultipart {
	fn intl(&mut self) -> &mut ResponseBuilderInternal {
		&mut self.intl_rb
	}
}
impl ResponseBuilderInterface for ResponseBuilder {}
impl ResponseBuilderInterface for ResponseBuilderMultipart {}


// REQUEST BUILDER INTERFACE
pub trait RequestBuilderInterfaceIntermediate {
	fn intl(&mut self) -> &mut RequestBuilderInternal;
}
pub trait RequestBuilderInterface: RequestBuilderInterfaceIntermediate {}

impl RequestBuilderInterfaceIntermediate for RequestBuilder {
	fn intl(&mut self) -> &mut RequestBuilderInternal {
		&mut self.intl_rb
	}
}
impl RequestBuilderInterfaceIntermediate for RequestBuilderMultipart {
	fn intl(&mut self) -> &mut RequestBuilderInternal {
		&mut self.intl_rb
	}
}
impl RequestBuilderInterface for RequestBuilder {}
impl RequestBuilderInterface for RequestBuilderMultipart {}


// MESSAGE BUILDER INTERFACE
trait MessageBuilderIntermediate {
	fn msg(&mut self) -> &mut MessageBuilder;
}
trait MessageBuilderMultipartIntermediate {
	fn msg(&mut self) -> &mut MessageBuilderMultipart;
}
pub trait MessageBuilderInterface: MessageBuilderIntermediate {}
pub trait MessageBuilderMultipartInterface: MessageBuilderMultipartIntermediate {}

impl MessageBuilderIntermediate for ResponseBuilder {
	fn msg(&mut self) -> &mut MessageBuilder {
		&mut self.intl_msg
	}
}
impl MessageBuilderMultipartIntermediate for ResponseBuilderMultipart {
	fn msg(&mut self) -> &mut MessageBuilderMultipart {
		&mut self.intl_msg
	}
}
impl MessageBuilderIntermediate for RequestBuilder {
	fn msg(&mut self) -> &mut MessageBuilder {
		&mut self.intl_msg
	}
}
impl MessageBuilderMultipartIntermediate for RequestBuilderMultipart {
	fn msg(&mut self) -> &mut MessageBuilderMultipart {
		&mut self.intl_msg
	}
}

impl MessageBuilderInterface for ResponseBuilder {}
impl MessageBuilderMultipartInterface for ResponseBuilderMultipart {}
