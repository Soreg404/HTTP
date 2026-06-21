pub mod collect_error;

mod parser;
mod message;

pub struct RequestCollector {
    msg: message::Message
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
}

