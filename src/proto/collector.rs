pub mod collect_error;
use collect_error::CollectError;

mod parser;
mod message;

pub struct RequestCollector {
    msg: message::Message
}
pub struct RequestFinished {
    msg: message::MessageFinished
}

impl RequestCollector {
    pub fn new() -> Self {
        Self {
            msg: message::Message::new_request()
        }
    }
    pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
        self.msg.push_bytes(bytes)
    }
    pub fn is_finished(&self) -> bool {
        self.msg.is_finished()
    }
    //pub fn to_finished_request(self) -> Result<RequestFinished, CollectError> {
        //self.message.to_finished()
    //}
}

