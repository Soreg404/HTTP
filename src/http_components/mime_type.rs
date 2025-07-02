use std::fmt::{Debug};

#[derive(Debug, Default, PartialEq, Ord, PartialOrd, Eq, Clone)]
pub enum MimeType {
	#[default]
	Unspecified,
	Multipart,
	TextPlain,
	TextHtml,
	TextJson,
	Image,
	ImagePng,
	ImageJpg
}
