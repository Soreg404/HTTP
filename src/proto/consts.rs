use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug)]
pub enum Method {
	GET,
	POST,
	PUT,
	PATCH,
	DELETE
}

impl FromStr for Method {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"GET" => Ok(Method::GET),
			"POST" => Ok(Method::POST),
			"PUT" => Ok(Method::PUT),
			"PATCH" => Ok(Method::PATCH),
			"DELETE" => Ok(Method::DELETE),
			_ => Err(())
		}
	}
}

#[derive(Debug)]
pub enum Version {
	HTTP09,
	HTTP10,
	HTTP11,
	HTTP20,
	HTTP30,
}

impl FromStr for Version {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"HTTP/0.9" => Ok(Version::HTTP09),
			"HTTP/1.0" => Ok(Version::HTTP10),
			"HTTP/1.1" => Ok(Version::HTTP11),
			"HTTP/2.0" => Ok(Version::HTTP20),
			"HTTP/3.0" => Ok(Version::HTTP30),
			_ => Err(())
		}
	}
}

impl Display for Version {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "HTTP/{}", match self {
			Version::HTTP09 => "0.9",
			Version::HTTP10 => "1.0",
			Version::HTTP11 => "1.1",
			Version::HTTP20 => "2.0",
			Version::HTTP30 => "3.0"
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
