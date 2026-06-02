
struct MessageCollector<T>
where T: MCSpec {
	pub spec_t: T
}

trait MCSpec {
	fn first_line();
}

struct RequestSpec {}

impl MCSpec for RequestSpec {
	fn first_line() {
		todo!()
	}
}

struct RequestCollector {
	msg: MessageCollector<RequestSpec>
}

trait MessageCollectorInterface: MessageCollectorInterfaceIntermediate {
	fn abc(&mut self) {
		self.msg();
	}
}

trait MessageCollectorInterfaceIntermediate {
	type MessageType: MCSpec;
	fn msg(&mut self) -> &mut MessageCollector<Self::MessageType>;
}

impl MessageCollectorInterfaceIntermediate for RequestCollector {
	type MessageType = RequestSpec;

	fn msg(&mut self) -> &mut MessageCollector<Self::MessageType> {
		todo!()
	}
}

impl RequestCollector {
	fn url(&self) -> ! {
		&self.msg.spec_t;
		todo!()
	}
}
impl MessageCollectorInterface for RequestCollector {}
