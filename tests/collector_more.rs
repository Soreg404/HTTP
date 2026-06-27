#[test]
fn take_target() {
    let sample = b"GET /target HTTP/1.1\r\n\r\n";
    let mut rc = http::RequestCollector::new();
    rc.push_bytes(sample);
    assert!(rc.is_finished());
    let req = rc.to_request().unwrap();


}
