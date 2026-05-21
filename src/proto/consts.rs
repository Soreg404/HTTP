use crate::consts::StatusCode::{IM_A_TEAPOT, NOT_FOUND, SUCCESS};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Method {
	UNKNOWN,
	GET,
	POST,
	PUT,
	PATCH,
	DELETE,
}

impl Method {
	pub fn from_bytes(bytes: &[u8]) -> Self {
		match bytes {
			b"GET" => Method::GET,
			b"POST" => Method::POST,
			b"PUT" => Method::PUT,
			b"PATCH" => Method::PATCH,
			b"DELETE" => Method::DELETE,
			_ => Method::UNKNOWN,
		}
	}
}

impl Display for Method {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		use Method::*;
		match self {
			GET => write!(f, "GET"),
			POST => write!(f, "POST"),
			PUT => write!(f, "PUT"),
			PATCH => write!(f, "PATCH"),
			DELETE => write!(f, "DELETE"),
			UNKNOWN => { write!(f, "<UNKNOWN>") }
		}
	}
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Version {
	HTTP_0_9,
	HTTP_1_0,
	HTTP_1_1,
	HTTP_2_0,
	HTTP_3_0,
}

impl Version {
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
		match bytes {
			b"HTTP/0.9" => Ok(Version::HTTP_0_9),
			b"HTTP/1.0" => Ok(Version::HTTP_1_0),
			b"HTTP/1.1" => Ok(Version::HTTP_1_1),
			b"HTTP/2.0" => Ok(Version::HTTP_2_0),
			b"HTTP/3.0" => Ok(Version::HTTP_3_0),
			_ => Err(())
		}
	}
}

impl Display for Version {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "HTTP/{}", match self {
			Version::HTTP_0_9 => "0.9",
			Version::HTTP_1_0 => "1.0",
			Version::HTTP_1_1 => "1.1",
			Version::HTTP_2_0 => "2.0",
			Version::HTTP_3_0 => "3.0"
		})
	}
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum StatusCode {
	SUCCESS = 200,
	NOT_FOUND = 404,
	IM_A_TEAPOT = 418,
}

impl StatusCode {
	pub fn as_desc(&self) -> &'static str {
		use StatusCode::*;
		match self {
			SUCCESS => "OK",
			NOT_FOUND => "NOT FOUND",
			IM_A_TEAPOT => "I'M A TEAPOT",
		}
	}
}

impl StatusCode {
	pub fn from_value(v: u32) -> Result<Self, ()> {
		match v {
			200 => Ok(SUCCESS),
			404 => Ok(NOT_FOUND),
			418 => Ok(IM_A_TEAPOT),
			_ => Err(())
		}
	}
}

#[derive(Clone, Debug)]
pub enum MimeType {
	Unspecified,
	Multipart,
	TextPlain,
	TextHtml,
	TextJson,
	Image,
	ImagePng,
	ImageJpg,
}
