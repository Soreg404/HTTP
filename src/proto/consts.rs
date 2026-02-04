use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::proto::parser::ParseError;

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

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Version {
	HTTP_0_9,
	HTTP_1_0,
	HTTP_1_1,
	HTTP_2_0,
	HTTP_3_0,
}

impl FromStr for Version {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"HTTP/0.9" => Ok(Version::HTTP_0_9),
			"HTTP/1.0" => Ok(Version::HTTP_1_0),
			"HTTP/1.1" => Ok(Version::HTTP_1_1),
			"HTTP/2.0" => Ok(Version::HTTP_2_0),
			"HTTP/3.0" => Ok(Version::HTTP_3_0),
			_ => Err(ParseError::InvalidVersion)
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

impl TryFrom<u32> for StatusCode {
	type Error = ParseError;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		use StatusCode::*;
		match value {
			200 => Ok(SUCCESS),
			404 => Ok(NOT_FOUND),
			418 => Ok(IM_A_TEAPOT),
			_ => Err(ParseError::InvalidStatusCode)
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
