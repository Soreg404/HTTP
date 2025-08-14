use std::fmt::Debug;

#[derive(Debug, Default, PartialEq, Ord, PartialOrd, Eq, Clone)]
pub enum MimeType {
	#[default]
	Unspecified,
	Multipart(String),
	TextPlain,
	TextHtml,
	TextJson,
	Image,
	ImagePng,
	ImageJpg,
}
