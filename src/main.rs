use std::sync::Arc;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

fn main() {
    let file = ask("file");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&Path::new(&file))
        .unwrap();
    let domain = ask("domain");
    let target = ask("target");

    let mut s = init_tls(domain.clone());
    let mut s = rustls::Stream::new(&mut s.0, &mut s.1);

    s.write_all(
        format!("GET {target} HTTP/1.1\r\n\
        host: {domain}\r\n\
        \r\n")
        .as_bytes()
    )
        .unwrap();

    let start = Instant::now();
    let mut last_measure = start;

    let mut rc = http::Collector::new();
    let mut msg_buf = Vec::new();
    let mut buf = Box::new([0u8; 64000]);
    loop {
        println!("before read");
        time(&start, &mut last_measure);
        let n = s.read(buf.as_mut()).unwrap();
        println!("after read");
        time(&start, &mut last_measure);
        msg_buf.extend_from_slice(&buf[..n]);
        let adv = rc.advance(&msg_buf[rc.get_proc_bytes()..]);
        match adv.av_action {
            http::AvAction::FirstLineReady(r) => {
                println!("first line ready: {:?}",
                    String::from_utf8_lossy(&msg_buf[r]));
            }
            http::AvAction::HeadersReady(r) => {
                println!("headers ready: <<<{}>>>",
                    String::from_utf8_lossy(&msg_buf[r]));
            }
            http::AvAction::BodyChunkReady(r) => {
                println!("chunk ready: [{}bytes]", r.end - r.start);
                file.write(&msg_buf[r])
                    .unwrap();
            }
            http::AvAction::BodyReady(r) => {
                println!("body ready: [{}bytes]", r.end - r.start);
                file.write(&msg_buf[r])
                    .unwrap();
            }
            _ => {}
        }
        if rc.finish_status().is_some() {
            println!("finished");
            break
        }
        if n == 0 && rc.get_proc_bytes() == msg_buf.len() {
            panic!("cant finish");
        }
    }

}

fn ask(about: &str) -> String {
    let mut s = String::new();
    print!("{about}: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut s).unwrap();
    String::from(s.trim())
}

fn time(start: &Instant, last_measure: &mut Instant) {
    let now = Instant::now();
    let since_start = now.duration_since(*start).as_secs_f64();
    let since_last = now.duration_since(*last_measure).as_secs_f64();
    *last_measure = now;
    println!("time: +{since_start}s / {since_last}s");
}

fn init_tls(dn: String)
-> (rustls::ClientConnection, std::net::TcpStream) {
    let root_store = rustls::RootCertStore::from_iter(
        webpki_roots::TLS_SERVER_ROOTS
        .iter()
        .cloned(),
    );

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let rc_config = Arc::new(config);

    let s = std::net::TcpStream::connect((dn.as_str(), 443))
        .unwrap();

    let c = rustls::ClientConnection::new(rc_config, dn.try_into().unwrap())
        .unwrap();

    (c, s)
}

