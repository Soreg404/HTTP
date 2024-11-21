fn main() {
    let mut rc = http::RequestCollector::new();

    let buffer = b"\
        GET /target HTTP/1.1\r\n\
        header1: haha\r\n\
        content-length: 20\r\n\
        \r\n\
        content content content content content content content";

    let mut dbg_chunks = buffer.chunks(11);

    for chunk in dbg_chunks {
        let pb = rc.process_bytes(chunk);
        println!("{pb:?}");
        match pb.state {
            http::ProcessBytesState::AllProcessed => {
                continue
            }
            http::ProcessBytesState::CanProcessHead => {
                rc.process_head(&buffer[..pb.processed_bytes_total]);
                rc.process_bytes(pb.remainder);
            }
            http::ProcessBytesState::Finished => {
                if !pb.remainder.is_empty() {
                    println!("finished with remainder");
                } else {
                    println!("finished without remainder");
                }
                break
            }
        }
    }

    println!("{rc:?}");

}
