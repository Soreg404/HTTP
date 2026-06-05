use super::glue::*;
use crate::proto::consts::Version;
use super::collect_error::CollectError;
use crate::proto::state_reader::StateReader;

#[derive(Default)]
pub struct Message<T>
where T: Subtype {
    state: CollectState,
    stage: CollectStage,
    buffer: Vec<u8>,
    buffer_reader: StateReader,
    specific: T,
    incomplete: MessageIncomplete
}

#[derive(Default)]
enum CollectState {
    #[default]
    Processing,
    Finished(Result<(), CollectError>)
}

#[derive(Default)]
enum CollectStage {
    #[default]
    FirstLine,
    MainHeaders,
    AfterMainHeaders,
    MainBody,
    Attachments
}

#[derive(Default)]
struct MessageIncomplete {
    headers: Vec<()>
}

enum AdvanceResult {
    Pending,
    Continue,
    ChangeStage(CollectStage),
    Finished(Result<(), CollectError>)
}
impl MessageIncomplete {
    fn advance(
        &mut self,
        buffer: &[u8],
        buffer_reader: &mut StateReader,
        stage: &CollectStage,
    ) -> AdvanceResult {
        AdvanceResult::Finished(Ok(()))
    }
}

impl<T> Message<T>
where T: Subtype + Default{
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        if let CollectStage::FirstLine = self.stage {
            let mut v = Version::HTTP_1_1;
            self.specific.first_line(
                b"GET / HTTP/1.1",
                &mut v
            );
        } 
        loop {
            match self.incomplete.advance(
                &self.buffer,
                &mut self.buffer_reader,
                &self.stage
            ) {
                AdvanceResult::Pending => break,
                AdvanceResult::Continue => continue,
                AdvanceResult::ChangeStage(s) => {
                    self.stage = s;
                    continue;
                }
                AdvanceResult::Finished(r) => {
                    self.state = CollectState::Finished(r);
                    break;
                }
            }
        }
        
    }
    pub fn is_finished(&self) -> bool {
        match self.state {
            CollectState::Processing => false,
            CollectState::Finished(_) => true
        }
    }
}
