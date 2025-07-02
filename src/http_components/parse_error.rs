#[derive(Debug)]
pub enum HTTPParseError {
	IncompleteRequest,
	IllegalByte,
	// maybe BodyTooLong,
	MalformedMessage(MalformedMessageKind)
}

#[derive(Debug)]
pub enum MalformedMessageKind {
	Other
}
