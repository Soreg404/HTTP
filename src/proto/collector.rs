pub struct RequestCollector {
    msg: RCMessage
}

type RCMessage = message::Message<subtypes::Request>;

pub mod collect_error;

mod parser;
mod glue;
mod message;
mod subtypes;
