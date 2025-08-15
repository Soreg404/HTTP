use super::{buffer_reader::BufferReader, message::HTTPMessage};
use crate::proto::internal::parser::{validate_http_line_bytes, FirstLineRequest, FirstLineResponse};
use crate::proto::internal::partial_message::AdvanceResult::{CanAdvanceMore, Finished};
use crate::proto::internal::partial_message::TransferEncoding::{Chunked, TillEOF, Unspecified};
use crate::proto::parse_error::HTTPParseError::MalformedMessage;
use crate::proto::parse_error::{HTTPParseError, MalformedMessageKind};
use crate::HTTPHeader;
use crate::MalformedMessageKind::{DuplicateTransferEncoding, MalformedHeader};
use std::io::Write;
use std::str::FromStr;
use HTTPParseError::IncompleteMessage;
use MalformedMessageKind::MalformedChunkTrailer;
use TransferEncoding::ContentLength;

#[derive(Default, PartialEq, Debug)]
enum ParseState {
	#[default]
	FirstLine,
	Headers,
	Body,
}

#[derive(Default, PartialEq, Debug)]
enum TransferEncoding {
	#[default]
	Unspecified,
	TillEOF,
	Chunked(Option<usize>),
	ContentLength(usize),
}

#[derive(Default)]
pub struct HTTPPartialMessage {
	parse_result: Option<Result<(), HTTPParseError>>,
	internal_buffer: BufferReader,

	state: ParseState,

	transfer_encoding: TransferEncoding,

	with_http_version: Option<(u8, u8)>,

	// todo: make it a reference to internal_buffer instead of owning
	with_headers: Vec<HTTPHeader>,
	with_body: Vec<u8>,
}

enum AdvanceResult {
	CanAdvanceMore,
	Finished,
}

impl HTTPPartialMessage {
	pub fn push_bytes(&mut self, data: &[u8]) {
		assert_eq!(self.parse_result, None);

		self.internal_buffer.write_all(data).unwrap();
	}
	pub fn advance(&mut self) {
		loop {
			match self.advance_state() {
				None => return,
				Some(Ok(ar)) => {
					match ar {
						CanAdvanceMore => continue,
						Finished => {
							self.parse_result = Some(Ok(()));
							return;
						}
					}
				}
				Some(Err(e)) => {
					self.parse_result = Some(Err(e));
					return;
				}
			}
		}
	}
	pub fn is_finished(&self) -> bool {
		self.parse_result.is_some()
	}
	pub fn signal_connection_closed(&mut self) {
		if self.parse_result.is_some() {
			return;
		}

		if match self.transfer_encoding {
			ContentLength(l) => self.with_body.len() == l,
			_ => false
		} {
			self.parse_result = Some(Ok(()));
		} else if self.state == ParseState::Body
			&& self.transfer_encoding == Unspecified {
			self.parse_result = Some(Ok(()));
		} else {
			self.parse_result = Some(Err(IncompleteMessage))
		}
	}

	///
	pub fn is_first_line(&self) -> bool {
		match self.state {
			ParseState::FirstLine => true,
			_ => false,
		}
	}
	pub fn take_first_line_request(&mut self) -> Option<FirstLineRequest> {
		assert_eq!(self.state, ParseState::FirstLine);
		let line = self.internal_buffer.take_line()?;
		self.state = ParseState::Headers;

		Some(
			match FirstLineRequest::try_from(line) {
				Ok(v) => v,
				Err(e) => {
					self.parse_result = Some(Err(e));
					return None;
				}
			}
		)
	}
	pub fn take_first_line_response(&mut self) -> Option<FirstLineResponse> {
		assert_eq!(self.state, ParseState::FirstLine);
		let line = self.internal_buffer.take_line()?;
		self.state = ParseState::Headers;

		Some(
			match FirstLineResponse::try_from(line) {
				Ok(v) => v,
				Err(e) => {
					self.parse_result = Some(Err(e));
					return None;
				}
			}
		)
	}

	///
	fn advance_state(&mut self) -> Option<Result<AdvanceResult, HTTPParseError>> {
		match self.state {
			ParseState::FirstLine => unreachable!("first line should be handled already"),
			ParseState::Headers => {
				let line = self.internal_buffer.take_line()?;

				let line = match validate_http_line_bytes(line) {
					Ok(line) => line,
					Err(e) => return Some(Err(e))
				};

				let line = line.trim();

				if line.is_empty() {
					self.state = ParseState::Body;
					return Some(Ok(CanAdvanceMore));
				}

				let header = match HTTPHeader::from_str(line) {
					Ok(header) => header,
					Err(e) => return Some(Err(e))
				};

				match header.name.to_lowercase().as_str() {
					"connection" => {
						if header.value == "close" {
							if self.transfer_encoding == Unspecified {
								self.transfer_encoding = TillEOF;
							}
						}
					}
					"transfer-encoding" => {
						match self.transfer_encoding {
							Unspecified | TillEOF => {
								self.transfer_encoding = Chunked(None);
							}
							Chunked(_) | ContentLength(_) => return Some(Err(
								MalformedMessage(DuplicateTransferEncoding)))
						}
					}
					"content-length" => {
						match self.transfer_encoding {
							Unspecified | TillEOF => {
								let value_parsed = match header.value
									.parse::<usize>() {
									Ok(value) => value,
									Err(e) => return Some(Err(
										MalformedMessage(MalformedHeader)))
								};

								self.transfer_encoding = ContentLength(value_parsed);
							}
							Chunked(_) | ContentLength(_) => return Some(Err(
								MalformedMessage(DuplicateTransferEncoding)))
						}
					}
					"content-length-range" => todo!("idk what is this header lol"),
					"content-type" => {
						println!("still can't parse content-type header!");
						// todo!("can't parse content-type header yet!")
					}
					_ => {}
				}

				self.with_headers.push(header.into());

				Some(Ok(CanAdvanceMore))
			}
			ParseState::Body => {
				match self.transfer_encoding {
					Unspecified => Some(Ok(Finished)),
					TillEOF => {
						let data = self.internal_buffer.take_all()?;
						Some(Ok(CanAdvanceMore))
					},
					Chunked(None) => {
						let line = self.internal_buffer.take_line()?;

						let line = match validate_http_line_bytes(line) {
							Ok(line) => line,
							Err(e) => return Some(Err(e))
						};

						let chunk_size = match usize::from_str_radix(line, 16) {
							Ok(chunk_size) => chunk_size,
							Err(e) => return Some(Err(
								MalformedMessage(MalformedChunkTrailer)))
						};

						if chunk_size == 0 {
							Some(Ok(Finished))
						} else {
							self.transfer_encoding = Chunked(Some(chunk_size));
							Some(Ok(CanAdvanceMore))
						}
					}
					Chunked(Some(length)) => {
						let data = self.internal_buffer.take_exact(length)?;

						self.with_body.extend(data);
						self.transfer_encoding = Chunked(None);
						Some(Ok(CanAdvanceMore))
					}
					ContentLength(length) => {
						let data = self.internal_buffer.take_exact(length)?;

						self.with_body = data.to_vec();

						Some(Ok(Finished))
					}
				}
			}
		}
	}
}

impl TryInto<HTTPMessage> for HTTPPartialMessage {
	type Error = HTTPParseError;

	fn try_into(self) -> Result<HTTPMessage, Self::Error> {
		if let Some(Err(e)) = self.parse_result {
			return Err(e);
		}

		Ok(
			HTTPMessage {
				http_version: self.with_http_version.unwrap_or((1, 0)),
				headers: self.with_headers,
				body: self.with_body,
			}
		)
	}
}
