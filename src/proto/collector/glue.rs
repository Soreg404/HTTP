use super::collect_error::CollectError;
use crate::proto::consts::Version;

pub trait Subtype {
    fn first_line(
        &mut self,
        line: &[u8],
        version: &mut Version
    ) -> Result<(), CollectError>;
}

impl super::RequestCollector {
    pub fn new() -> Self {
        Self {
            msg: super::RCMessage::new()
        }
    }
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.msg.push_bytes(bytes)
    }
    pub fn is_finished(&self) -> bool {
        self.msg.is_finished()
    }
}
