#[test]
fn basic_api() {
    let mut rc = http::RequestCollector::new();

    let first_line = b"GET /target HTTP/1.1\r\n";
    let content = b"hello world!";
    let headers_str = format!(
        "content-length: {}\r\n\
        x-sample-header:     header value    \r\n\
        \r\n",
        content.len()
    );
    let headers = headers_str.as_bytes();

    let n = rc.push_bytes(first_line);
    assert_eq!(n, first_line.len());
    assert!(!rc.is_finished());

    let n = rc.push_bytes(headers);
    assert_eq!(n, headers.len());
    assert!(!rc.is_finished());

    let n = rc.push_bytes(content);
    assert_eq!(n, content.len());
    assert!(rc.is_finished());
}

#[test]
fn to_finished() {
    let sample = b"\
    GET /target HTTP/1.1\r\n\
    content-length: 5\r\n\
    \r\n\
    12345";

    let rc = {
        let mut tmp_rc = http::RequestCollector::new();
        tmp_rc.push_bytes(sample);
        tmp_rc
    };

    let req = rc.to_request();
    assert!(req.is_ok());
}

