#[derive(Debug, Eq, PartialEq)]
pub enum HTTPParseError {
	IncompleteMessage,
	IllegalByte,
	// maybe BodyTooLong,
	MalformedMessage(MalformedMessageKind),
	MissingContentTypeHeader,
	MissingMultipartBoundary,
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
	MalformedHeader,
	DuplicateTransferEncoding,
	MalformedChunkTrailer,
	MalformedHTTPVersion,
	InvalidContentDisposition,
	MalformedMultipartBody,
}
