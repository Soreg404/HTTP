mod into_message;

use crate::proto::buffer_reader::{DelayedConsumeResult, DelayedStateBuffer};
use crate::proto::parser;
use crate::proto::parser::{HeaderLineParseResult, ParseError};


pub type CollectResult = Option<Result<(), ParseError>>;

#[derive(Copy, Clone, Default)]
enum CollectPhase {
	#[default]
	FirstLine,
	MainHeaders,
	MainBody,
}

#[derive(Copy, Clone)]
pub enum CollectorState {
	Incomplete(CollectPhase),
	Finished(Result<(), ParseError>),
}

pub struct MessageCollector {
	collector_state: CollectorState,

	collected_headers: Vec<(Vec<u8>, Vec<u8>)>,
	collected_body: Vec<u8>,

	master_buffer_reader: DelayedStateBuffer,
}

impl MessageCollector {
	pub fn new() -> Self {
		Self {
			collector_state: CollectorState::Incomplete(Default::default()),

			collected_headers: vec![],
			collected_body: vec![],

			master_buffer_reader: DelayedStateBuffer::new(),
		}
	}
}

pub enum MessageCollectorAdvance {
	NeedMoreBytes,
	Finished {
		remaining_bytes: usize
	},
	Error(ParseError),
}

enum AdvanceSingleResult {
	CanContinue,
	ChangePhase(CollectPhase),
	NotEnoughBytes,
	Finished,
	Error(ParseError),
}

impl MessageCollector {
	pub fn is_finished(&self) -> bool {
		use CollectorState::*;
		match &self.collector_state {
			Incomplete(_) => false,
			Finished(_) => true
		}
	}

	pub fn advance<F>(&mut self, buffer: &[u8], mut on_first_line: F)
					  -> MessageCollectorAdvance
	where
		F: FnMut(&[u8]) -> Result<(), ParseError>,
	{
		use AdvanceSingleResult::*;

		if let CollectorState::Incomplete(CollectPhase::FirstLine) = self.collector_state {
			match self.master_buffer_reader.take_line(buffer) {
				DelayedConsumeResult::NotEnoughBytes =>
					return MessageCollectorAdvance::NeedMoreBytes,
				DelayedConsumeResult::Finished { slice, .. } => {
					match on_first_line(slice) {
						Ok(()) => {
							self.collector_state =
								CollectorState::Incomplete(
									CollectPhase::MainHeaders);
						}
						Err(e) => {
							return MessageCollectorAdvance::Error(e)
						}
					}
				}
			};
		}

		loop {
			match self.advance_single(buffer) {
				CanContinue => continue,
				ChangePhase(p) => {
					self.collector_state = CollectorState::Incomplete(p);
					continue;
				}
				NotEnoughBytes => {
					return MessageCollectorAdvance::NeedMoreBytes
				}
				Finished => {
					self.collector_state = CollectorState::Finished(Ok(()));
					return MessageCollectorAdvance::Finished {
						remaining_bytes: buffer.len() - self.master_buffer_reader.consumed(),
					};
				}
				Error(e) => {
					self.collector_state = CollectorState::Finished(Err(e));
					return MessageCollectorAdvance::Error(e);
				}
			}
		}
	}

	fn advance_single(&mut self, buffer: &[u8]) -> AdvanceSingleResult {
		use CollectPhase::*;
		use DelayedConsumeResult::*;
		use AdvanceSingleResult as ADV;

		match self.collector_state {
			CollectorState::Finished(_) => unreachable!(),
			CollectorState::Incomplete(phase) => match phase {
				FirstLine => unreachable!(),
				MainHeaders => {
					match self.master_buffer_reader.take_line(&buffer) {
						NotEnoughBytes => ADV::NotEnoughBytes,
						Finished { slice, .. } => {
							match parser::parse_header_line(slice) {
								HeaderLineParseResult::Empty => {
									ADV::ChangePhase(MainBody)
								}
								HeaderLineParseResult::Err(e) => {
									dbg!(e, &self.master_buffer_reader);
									dbg!(String::from_utf8_lossy(slice));
									ADV::Error(e)
								}
								HeaderLineParseResult::Ok {
									field_name,
									field_value
								} => {
									self.collected_headers
										.push((
											field_name.to_owned(),
											field_value.to_owned(),
										));
									ADV::CanContinue
								}
							}
						}
					}
				}
				MainBody => {
					dbg!(&self.collected_headers);
					ADV::Finished
				}
			}
		}
	}
}
