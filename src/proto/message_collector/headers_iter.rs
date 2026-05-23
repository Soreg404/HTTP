use crate::proto::header_parser::HeaderRdx;
use crate::proto::rdx::Rdx;

pub struct HeadersIter<'a> {
	buffer: &'a [u8],
	headers: &'a [HeaderRdx],
	head: usize
}
pub struct Header<'a> {
	field_name: &'a [u8],
	field_body: &'a [u8],
}
impl<'a> Iterator for HeadersIter<'a> {
	type Item = Header<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.head == self.headers.len() {
			None
		} else {
			let r = Some(Header {
				field_name: self.headers[self.head].name.get(self.buffer),
				field_body: self.headers[self.head].body.get(self.buffer),
			});
			self.head += 1;
			r
		}
	}
}

#[test]
fn a_headers_iter() {
	let buffer = b"header1: body\r\nheader2: other\r\n";
	let headers_rdx_vec = [
		HeaderRdx {
			name: Rdx::new(0, 7),
			body: Rdx::new(9, 13),
		},
		HeaderRdx {
			name: Rdx::new(15, 22),
			body: Rdx::new(24, 29),
		}
	];

	let mut it = HeadersIter {
		buffer,
		headers: &headers_rdx_vec,
		head: 0,
	};

	match it.next() {
		None => panic!(),
		Some(h) => {
			assert_eq!(h.field_name, b"header1");
			assert_eq!(h.field_body, b"body");
		}
	}
	match it.next() {
		None => panic!(),
		Some(h) => {
			assert_eq!(h.field_name, b"header2");
			assert_eq!(h.field_body, b"other");
		}
	}

	assert!(it.next().is_none());
}
