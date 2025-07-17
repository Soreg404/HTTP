pub fn check_ends_with_new_line(bytes: &[u8]) -> bool {
	bytes.last() == Some(&b'\n')
}

pub fn strip_last_endl(bytes: &[u8]) -> &[u8] {
	let Some(bytes) = bytes.strip_suffix(b"\n") else { return bytes };
	let Some(bytes) = bytes.strip_suffix(b"\r") else { return bytes };
	bytes
}
