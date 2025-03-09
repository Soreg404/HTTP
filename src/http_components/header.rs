pub struct HTTPHeader {
	pub name: String,
	pub value: String,
}
impl HTTPHeader {
	pub fn new(name: impl Into<String>, value: impl Into<String>) -> HTTPHeader {
		HTTPHeader { name: name.into(), value: value.into() }
	}

	pub fn parse_value(&self) -> Vec<(String, Vec<(String, String)>)> {
		unimplemented!();

		// let mut in_quote = false;
		// let parts = self.value.split(|c| {
		// 	if c == '"' {
		// 		in_quote = !in_quote;
		// 	}
		// 	if in_quote { return false; }
		// 	if c == ',' {
		// 		true
		// 	} else {
		// 		false
		// 	}
		// });
		//
		// let mut ret: Vec<Vec<String>> = Vec::new();
		// for part in parts {
		// 	ret.push()
		// }

	}
}
