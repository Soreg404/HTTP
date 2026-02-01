
pub mod parse_error;
pub type HTTPParseResult<T> = Result<T, parse_error::HTTPParseError>;
