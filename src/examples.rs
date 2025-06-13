

pub fn run_examples() {

	blue("\n# examples\n");

	example(example_compose_request, "compose / create a POST request");

	example(example_compose_response, "compose / create a response");

	example(example_quick_response, "quick response");

	example(example_read_request, "read request");
}

fn green(text: &str) {
	println!("\x1b[92m{text}\x1b[0m");
}
fn blue(text: &str) {
	println!("\x1b[94m{text}\x1b[0m");
}

fn example(which: fn(), title: &str) {

	let header = format!("## example: {title}");

	blue(format!("{header}").as_str());

	which();

	blue(format!("{}", "-".repeat(header.len())).as_str());
	println!()
}

fn example_compose_request() {
	let mut req = http::HTTPRequest::default();
	req.method = String::from("POST");
	req.url.path_raw = String::from("/api/get-list");
	req.url.query_string_raw = String::from("hello=world");
	req.headers.push(http::HTTPHeader {
		name: String::from("host"),
		value: String::from("localhost"),
	});
	req.headers.push(http::HTTPHeader {
		name: String::from("content-type"),
		value: String::from("text/txt"),
	});
	req.body = "Lorem ipsum\ndolor sit amet".as_bytes().to_vec();

	green("[debug:?]");
	println!("{req:?}");
	green("[display]");
	println!("{req}");
}

fn example_compose_response() {
	let mut res = http::HTTPResponse::default();
	res.status = 200;
	res.headers.push(http::HTTPHeader {
		name: String::from("content-type"),
		value: String::from("text/json"),
	});
	res.body = r#"{"result": "works", "number": 42}"#.as_bytes().to_vec();


	green("[debug:?]");
	println!("{res:?}");
	green("[display]");
	println!("{res}");
}

fn example_quick_response() {
	println!("created with ..default: {}", http::HTTPResponse {
		status: 418,
		..http::HTTPResponse::default()
	});

	println!("created with quick: {}", http::HTTPResponse::quick(404));
}

fn example_read_request() {
	let mut p_req = http::HTTPPartialRequest::default();

	p_req.push_bytes(b"POST /fi");
	p_req.push_bytes(b"nd HTTP/1.1\r");
	p_req.push_bytes(b"\nhost: local");
	p_req.push_bytes(b"host\r\n");
	p_req.push_bytes(b"content-length: 5\r\n\r\n");
	p_req.push_bytes(b"Hello");
	p_req.push_bytes(b"this won't be saved to request body (bsc content-length)");

	println!("partial request (is_complete={})", p_req.is_complete());
	green("[debug:?]");
	println!("{p_req:?}");
}
