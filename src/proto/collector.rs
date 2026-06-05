pub struct RequestCollector {
    msg: RCMessage
}

type RCMessage = message::Message<subtypes::Request>;

pub mod collect_error;

mod glue;
mod message;
mod subtypes;
