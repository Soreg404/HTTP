pub mod collect_error;

mod parser;
mod message;

use collect_error::CollectError;
use crate::proto::consts::Method;

pub struct RequestCollector {
    msg: message::Message
}
pub struct RequestFinished {
    method: Method,
    target: Vec<u8>,
    msg: message::MessageFinished,
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
    pub fn to_request(self) -> Result<RequestFinished, CollectError> {
        let mut fm = self.msg.to_finished()?;
        assert!(fm.f_request.is_some());
        assert!(fm.f_response.is_none());
        let f_req = fm.f_request.take().unwrap();
        Ok(RequestFinished {
            method: f_req.method,
            target: f_req.target,
            msg: fm
        })
    }
}
