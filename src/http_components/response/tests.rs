use super::*;

#[test]
fn abc() {
	let quick_response = HTTPResponse::quick(418);

	assert_eq!(quick_response.status, 418);
}
