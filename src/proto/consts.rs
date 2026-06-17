use std::fmt::{Debug, Display, Formatter};

#[derive(Default, Debug)]
pub enum Method {
    #[default]
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
		use self::Method::*;
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
#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub enum Version {
	HTTP_0_9,
	HTTP_1_0,
    #[default]
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
impl Debug for Version {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		<Self as Display>::fmt(self, f)
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
	pub fn as_desc(&self) -> &'static [u8] {
		use self::StatusCode::*;
		match self {
			SUCCESS => b"OK",
			NOT_FOUND => b"NOT FOUND",
			IM_A_TEAPOT => b"I'M A TEAPOT",
		}
	}
}

impl StatusCode {
	pub fn from_value(v: u32) -> Result<Self, ()> {
		match v {
			200 => Ok(StatusCode::SUCCESS),
			404 => Ok(StatusCode::NOT_FOUND),
			418 => Ok(StatusCode::IM_A_TEAPOT),
			_ => Err(())
		}
	}
}

#[derive(Clone, Debug, Default)]
pub enum MimeType {
	#[default]
	TextPlain,
	TextHtml,
	TextJson,
	Image,
	ImagePng,
	ImageJpg,
	MultipartFormData,
}
