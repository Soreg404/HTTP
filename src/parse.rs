use super::*;

impl RequestCollector {
    pub fn should_call_parse_head(&self) -> bool {
        match self.stage {
            CollectStage::ParseHead => true,
            _ => false
        }
    }
    pub fn is_done(&self) -> bool {
        match self.stage {
            CollectStage::Done => true,
            _ => false
        }
    }
    pub fn parse_head(&mut self, head_bytes: &[u8]) {

    }
}
