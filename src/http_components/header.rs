pub struct HTTPHeader {
	pub name: String,
	pub value: String,
}
impl HTTPHeader {
	pub fn new(name: impl Into<String>, value: impl Into<String>) -> HTTPHeader {
		HTTPHeader { name: name.into(), value: value.into() }
	}
}
