#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ParseError {
	TBD(&'static str),
	FirstLine,
	HeaderLine,
	InvalidStatusCode,
	InvalidVersion,
}
