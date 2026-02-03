mod into_message;

use crate::proto::buffer_reader::{DelayedConsumeResult, DelayedStateBuffer};
use crate::proto::parser;
use crate::proto::parser::{ParseError, ParseResult};


#[derive(Copy, Clone)]
enum CollectPhase {
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
			collector_state: CollectorState::Incomplete(CollectPhase::MainHeaders),

			collected_headers: vec![],
			collected_body: vec![],

			master_buffer_reader: DelayedStateBuffer::new(),
		}
	}
}

pub struct Advance {
	pub consumed: usize,
	pub collector_state: CollectorState,
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

	pub fn advance(&mut self, buffer: &[u8]) -> Advance {
		dbg!(String::from_utf8_lossy(&buffer[0..16]));
		use AdvanceSingleResult::*;

		let advance_start_read_head =
			self.master_buffer_reader.current_read_head();

		loop {
			match self.advance_single(buffer) {
				CanContinue => continue,
				ChangePhase(p) => {
					self.collector_state = CollectorState::Incomplete(p);
					continue;
				}
				NotEnoughBytes => {
					return Advance {
						consumed: buffer.len(),
						collector_state: self.collector_state,
					}
				}
				Finished => {
					self.collector_state = CollectorState::Finished(Ok(()));
					return Advance {
						consumed: self.master_buffer_reader
							.current_read_head() - advance_start_read_head,
						collector_state: self.collector_state,
					};
				}
				Error(e) => {
					self.collector_state = CollectorState::Finished(Err(e));
					return Advance {
						consumed: 0,
						collector_state: self.collector_state,
					};
				}
			}
		}
	}

	fn advance_single(&mut self, buffer: &[u8]) -> AdvanceSingleResult {
		use CollectPhase::*;
		use DelayedConsumeResult::*;
		use AdvanceSingleResult as ADV;

		match self.collector_state {
			CollectorState::Finished(_) => ADV::Finished,
			CollectorState::Incomplete(phase) => match phase {
				MainHeaders => {
					dbg!(&self.master_buffer_reader);
					match self.master_buffer_reader.take_line(&buffer) {
						NotEnoughBytes => ADV::NotEnoughBytes,
						Finished { slice, .. } => {
							match parser::parse_header_line(slice) {
								ParseResult::Empty => {
									ADV::ChangePhase(MainBody)
								}
								ParseResult::Err(e) => {
									dbg!(e, &self.master_buffer_reader);
									dbg!(String::from_utf8_lossy(slice));
									ADV::Error(e)
								}
								ParseResult::Ok {
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
