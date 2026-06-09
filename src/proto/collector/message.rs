use super::glue::*;
use crate::proto::consts::Version;
use super::collect_error::CollectError;
use crate::proto::state_reader::{ StateReader, Poll };

impl super::RequestCollector {
    pub fn debug_state(&self) -> String {
        format!("{:?}", self.msg.state)
    }
}


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

#[derive(Default, Debug)]
enum CollectState {
    #[default]
    Processing,
    Finished(Result<(), CollectError>)
}

#[derive(Default, Debug)]
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
    version: Version,
    headers: Vec<()>,
}

enum AdvanceResult {
    Pending,
    Continue,
    ChangeStage(CollectStage),
    Finished(Result<(), CollectError>)
}

impl<T> Message<T>
where T: Subtype + Default {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        // temporary
        self.buffer.extend_from_slice(bytes);
        ////
        if let CollectStage::FirstLine = self.stage {
            match self.buffer_reader.take_line(
                &self.buffer
            ) {
                Poll::Pending => return,
                Poll::Ready(line_rdx) => {
                    match self.specific.first_line(
                        line_rdx.get(&self.buffer),
                        &mut self.incomplete.version 
                    ) {
                        Ok(()) => {
                            self.stage = CollectStage::MainHeaders;
                        },
                        Err(e) => {
                            self.state = CollectState::Finished(Err(e));
                            return;
                        }
                    }
                }
            }
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

impl MessageIncomplete {
    fn advance(
        &mut self,
        buffer: &[u8],
        buffer_reader: &mut StateReader,
        stage: &CollectStage,
    ) -> AdvanceResult {
        match stage {
            CollectStage::FirstLine => unreachable!(),
            CollectStage::MainHeaders => {
                let line_rdx = match buffer_reader.take_line(&buffer) {
                    Poll::Pending => return AdvanceResult::Pending,
                    Poll::Ready(v) => v
                };

                let line_bytes = line_rdx.get(&buffer);
                dtrace!("MainHeaders", format!("processing line: {:?}",
                    String::from_utf8_lossy(line_bytes)));

                if line_bytes.trim_ascii().is_empty() {
                    if !line_bytes.is_empty() {
                        return AdvanceResult::Finished(Err(
                                CollectError::TBD("invalid empty header line".to_string())));
                    }
                    dtrace!("Mainheaders", "empty header line, \
                        change stage to AfterMainHeaders");
                    return AdvanceResult::ChangeStage(
                        CollectStage::AfterMainHeaders);
                }

                dtrace!("MainHeaders", "continue MainHeaders");
                AdvanceResult::Continue
            },
            _ => AdvanceResult::Finished(Ok(()))
        }
    }
}
