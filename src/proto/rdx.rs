#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Rdx {
	from: usize,
	to: usize,
}

impl Rdx {
	pub fn new(from: usize, to: usize) -> Self {
		assert!(from <= to, "Rdx constraint failed: from={from} > to={to}");
		Self { from, to }
	}
	pub fn offset(self, base: usize) -> Self {
		Self {
			from: self.from + base,
			to: self.to + base,
		}
	}
	pub fn from(&self) -> usize { self.from }
	pub fn to(&self) -> usize { self.to }
	pub fn get<'a>(&self, from: &'a [u8]) -> &'a [u8] {
		&from[self.from..self.to]
	}
	pub fn len(&self) -> usize {
		self.to - self.from
	}
}
