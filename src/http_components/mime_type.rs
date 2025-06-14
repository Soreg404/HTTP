use std::fmt::{Debug};

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq)]
pub enum MimeType {
	Unspecified,
	Multipart(String),
	TextPlain,
	TextHtml,
	TextJson,
	Image,
	ImagePng,
	ImageJpg
}
