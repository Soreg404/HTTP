pub struct Header<'a> {
	pub field_name: &'a [u8],
	pub field_body: &'a [u8]
}

pub struct HeaderRdx {
	pub field_name: Rdx,
	pub field_body: Rdx,
}
