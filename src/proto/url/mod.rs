#[derive(Default, Clone)]
pub struct Url {
	// [scheme://][domain][:port]/[path][?query_string][#fragment]
	pub scheme: Option<String>,
	pub domain: Option<String>,
	pub port: Option<u16>,
	pub path: String,
	pub query_string: String,
	pub fragment: String,
}

impl Url {
	/** Very much todo! */
	fn from_target(target: &[u8]) -> Option<Self> {
		let mut query_pos: Option<usize> = None;
		for (i, c) in target.iter().cloned().enumerate() {
			if c == b'?' {
				query_pos = Some(i);
			}
		}
		if query_pos.is_some() {
			let pos = query_pos.unwrap();
			Some(Self {
				path: String::from_utf8_lossy(&target[..pos]).to_string(),
				query_string: String::from_utf8_lossy(&target[pos..]).to_string(),
				..Self::default()
			})
		} else {
			Some(Self{
				path: String::from_utf8_lossy(target).to_string(),
				..Self::default()
			})
		}
	}
}
