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

#[allow(non_camel_case_types)]
pub enum StatusCode {
	SUCCESS = 200,
	NOT_FOUND = 404,
	IM_A_TEAPOT = 418,
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
