use crate::consts::Version;
use crate::proto::parser::ParseError;
use crate::proto::tmp_http_header::Header;

trait MessageCommonCollectorFirstLine {
	fn parse_first_line(&mut self, line_bytes: &[u8]) -> Result<Version, ParseError>;
}

pub struct MessageCommonCollector<T>
where T: MessageCommonCollectorFirstLine
{
	message_specific: T,
	version: Option<Version>,
	headers: Vec<Header>,
	body: Option<Vec<u8>>,

	multipart_boundary: Option<Vec<u8>>,
	multipart_attachments: Vec<Attachment>
}

struct Attachment {
	headers: Vec<Header>,
	name: Vec<u8>
}
