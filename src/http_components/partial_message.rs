
enum ParsePart {
	MainHeaders,
	MainBody,

	MultipartStart,
	AttachmentHeaders,
	AttachmentBody,
	MultipartEnd,
}

pub struct PartialMessage {

}
