pub struct Header {
	name: Vec<u8>,
	value: Vec<u8>,
}

impl Header {
	fn new(name: &[u8], value: &[u8]) -> Self {
		Self {
			name: name.into(),
			value: value.into()
		}
	}
	fn name(&self) -> &[u8] {
		&self.name
	}
	fn value(&self) -> &[u8] {
		&self.value
	}
}
