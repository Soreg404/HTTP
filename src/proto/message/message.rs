mod collector;
mod builder;

pub use collector::*;
pub use builder::*;

#[derive(Debug)]
pub struct Message {
	headers: Vec<(String, String)>,
	body: Vec<u8>,
}
