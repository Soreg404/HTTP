#[derive(Debug, Eq, PartialEq)]
pub enum HTTPParseError {
	IncompleteRequest,
	IllegalByte,
	// maybe BodyTooLong,
	MalformedMessage(MalformedMessageKind)
}

#[derive(Debug, Eq, PartialEq)]
pub enum MalformedMessageKind {
	Other,
	MimetypeMissingBoundaryParam,
	MimetypeParamMissingEqualSign,
	MultipartFirstLineBoundary,
	HeaderContentDisposition,
	MultipartEndsWithInvalidBytes,
	FirstLine,
	UrlGeneral,
	FirstLineStatusCode,
}
