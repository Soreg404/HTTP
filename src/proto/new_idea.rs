fn start() {
	let mut arena = Vec::new();
	arena.resize(0x400, 0u8);
}

#[derive(Clone)]
enum CState {
	Processing(CStage),
	Ended(Result<(), CollectError>),
}

#[derive(Clone)]
enum CollectError {
	TBD,
	MessageTooBig,
	IllegalByte,
}

#[derive(Clone)]
enum CStage {
	FirstLineReq(CFirstLineReq),
	FirstLineRes(CFirstLineRes),
}

#[derive(Clone)]
enum CFirstLineReq {
	Method,
	Url,
	Version,
}

#[derive(Clone)]
enum CFirstLineRes {
	Version,
	StatusCode,
	StatusDesc,
}

struct MessageCollector<'a> {
	state: CState,

	arena: &'a mut [u8],
	arena_base: usize,
	arena_head: usize,

	skip_space: bool,
}

enum TakeResult<T> {
	More,
	Done(T),
	Error(CollectError),
}

impl MessageCollector<'_> {
	fn combine_arena_take_until_space<'a, 'b>(&'a mut self, buffer: &'b [u8])
									   -> TakeResult<(&'a [u8], &'b [u8])> {
		let mut i = 0;
		while i < buffer.len() {
			if self.arena_head == self.arena.len() {
				return TakeResult::Error(CollectError::MessageTooBig);
			}
			if !is_legal_byte(buffer[i]) {
				return TakeResult::Error(CollectError::IllegalByte);
			}

			if buffer[i] == b' ' {
				let base = self.arena_base;
				self.arena_base = self.arena_head;
				return TakeResult::Done(
					(
						&self.arena[base..self.arena_head],
						&buffer[i..]
					)
				);
			} else {
				self.arena[self.arena_head] = buffer[i];
				self.arena_head += 1;
				i += 1;
			}
		}
		TakeResult::More
	}

	fn skip_single_space(&mut self, buffer: &[u8]) -> TakeResult<&[u8]> {
		if buffer.is_empty() {
			return TakeResult::More;
		}
		if buffer[0] == b' ' {
			if self.skip_space {
				return TakeResult::Error(CollectError::TBD);
			}
			let buffer = &buffer[1..];
			if buffer.is_empty() {
				return TakeResult::More;
			}
			if buffer[0] == b' ' {
				TakeResult::Error(CollectError::TBD)
			} else {
				TakeResult::Done(&buffer[1..])
			}
		} else {
			TakeResult::Error(CollectError::IllegalByte)
		}
	}
}

struct RequestCollector<'a> {
	mc: MessageCollector<'a>,
}

fn is_legal_byte(b: u8) -> bool {
	true
}

impl RequestCollector<'_> {
	fn request_push_bytes(&mut self, bytes: &[u8]) -> usize {
		match self.mc.state.clone() {
			CState::Ended(_) => return 0,
			CState::Processing(s) => {
				match s {
					CStage::FirstLineReq(_) => {
						loop {
							match self.mc.state.clone() {
								CState::Processing(CStage::FirstLineReq(s)) => {
									match s {
										CFirstLineReq::Method => {
											let mut i = 0;
											loop {
												if self.mc.arena_head

												if !is_legal_byte(bytes[i]) {
													self.mc.state = CState::Ended(Err(CollectError::IllegalByte));
													return i;
												}


												i += 1;
											}
										}
										CFirstLineReq::Url => {}
										CFirstLineReq::Version => {}
									}
								}
								_ => unreachable!()
							}
						}
					}
					s => {
						message_common_push_bytes(bytes)
					}
				}
			}
		}
	}
}
