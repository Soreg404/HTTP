use super::message::HTTPMessage;

pub trait GetMessageRefInternal {
	fn get_message(&self) -> &HTTPMessage;

	// todo: check spelling: get_message_mut or get_mut_message?
	fn get_message_mut(&mut self) -> &mut HTTPMessage;
}
