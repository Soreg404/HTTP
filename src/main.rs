#[path = "../samples.rs"]
mod samples;

fn main() {
    let mut rc = http::RequestCollector::new();
    rc.push_bytes(b"GET / HTTP/1.1\r\n\r\n");
    assert!(rc.is_finished());
}

