#[path = "../samples.rs"]
mod samples;

fn main() {
    let mut rc = http::RequestCollector::new();
    rc.push_bytes(b"\
        GET / HTTP/1.1\r\n\
        header1: value1\r\n\
        \r\n");
    println!("rc.debug_state(): {}", rc.debug_state());
}

