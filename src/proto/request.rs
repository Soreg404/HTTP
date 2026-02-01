use crate::proto::collector::{Collector, CollectorInterface, GetCollector, OnFirstLine};
use crate::proto::http_method::HTTPMethod;
use crate::proto::url::Url;

pub struct HTTPRequest {

}

pub struct HTTPRequestCollector {
	method: HTTPMethod,
	target: Url,
	collector: Collector
}

impl GetCollector for HTTPRequestCollector {
	fn get_collector(&self) -> &Collector {
		&self.collector
	}

	fn get_collector_mut(&mut self) -> &mut Collector {
		&mut self.collector
	}
}

impl OnFirstLine for HTTPRequestCollector {
	fn on_first_line(&mut self, line: &[u8]) {
		// method, target & version
	}
}

impl CollectorInterface for HTTPRequestCollector {}
