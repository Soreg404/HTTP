pub enum HTTPParseError {
	MalformedFirstLine,
	UnsupportedHTTPVersion,
	MalformedHeader,
}
