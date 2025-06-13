#[derive(PartialEq, Ord, PartialOrd, Eq)]
pub enum MimeType {
	Multipart,
	TextPlain,
	TextHtml,
	TextJson,
	Image,
	ImagePng,
	ImageJpg
}
