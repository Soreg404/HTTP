use crate::HTTPParseError;
use crate::HTTPParseError::IllegalByte;
use std::ops::Deref;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HTTPAsciiStr<'a> {
	inner: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HTTPAsciiString {
	inner: String,
}

impl<'a> TryFrom<&'a [u8]> for HTTPAsciiStr<'a> {
	type Error = HTTPParseError;

	fn try_from(value_bytes: &'a [u8]) -> Result<HTTPAsciiStr<'a>, Self::Error> {
		for b in value_bytes.iter().cloned() {
			if !b.is_ascii_graphic() && b != b' ' {
				return Err(IllegalByte);
			}
		}
		Ok(
			unsafe {
				HTTPAsciiStr {
					inner: str::from_utf8_unchecked(value_bytes)
				}
			}
		)
	}
}


impl<'a> TryFrom<&'a str> for HTTPAsciiStr<'a> {
	type Error = HTTPParseError;

	fn try_from(value: &'a str) -> Result<HTTPAsciiStr<'a>, Self::Error> {
		Self::try_from(value.as_bytes())
	}
}

impl Deref for HTTPAsciiStr<'_> {
	type Target = str;
	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl TryFrom<&[u8]> for HTTPAsciiString {
	type Error = HTTPParseError;
	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		Ok(
			Self {
				inner: HTTPAsciiStr::try_from(value)?.inner.to_string()
			}
		)
	}
}

impl TryFrom<&str> for HTTPAsciiString {
	type Error = HTTPParseError;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Self::try_from(value.as_bytes())
	}
}

impl TryFrom<String> for HTTPAsciiString {
	type Error = HTTPParseError;
	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_bytes())
	}
}

impl Deref for HTTPAsciiString {
	type Target = String;
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
