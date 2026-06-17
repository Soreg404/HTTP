#[path = "../samples.rs"]
mod samples;

static mut EXAMPLE_COUNTER: usize = 1;

macro_rules! ex_trailer { ($title:expr) => { 
    if unsafe { EXAMPLE_COUNTER } > 1 {
        print!("\n\n");
    }
    println!("\x1b[96m=== example {}: {}\x1b[0m", unsafe { EXAMPLE_COUNTER }, $title);
    unsafe { EXAMPLE_COUNTER += 1; }
} }

fn main() {

    ex_trailer!("simple");
        let mut rc = http::RequestCollector::new();
    rc.push_bytes(b"\
        GET / HTTP/1.1\r\n\
        header1: value1\r\n\
        \r\n");

    ex_trailer!("sample multipart");
    let mut rc = http::RequestCollector::new();
    rc.push_bytes(b"\
        GET / HTTP/1.1\r\n\
        content-length: 94\r\n\
        content-type: multipart/form-data; boundary=\"ABC\"\r\n\
        \r\n\
        --ABC\r\n\
        content-disposition: name=\"field1\"\r\n\
        content-type: text/plain\r\n\
        \r\n\
        hello world!
        --ABC--\r\n\
        ");
}

