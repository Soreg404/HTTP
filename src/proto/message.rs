pub struct MessageCollector<T>
where T: CollectFirstLine {
	collect_t: T,
}

trait CollectFirstLine {
	fn collect_first_line(&mut self, line: &[u8]);
}

pub struct RequestCollectorSpecific {}
pub struct ResponseCollectorSpecific {}
