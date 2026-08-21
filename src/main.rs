fn main() {
    let sample = b"\
    HTTP/1.1 200 OK\r\n\
    content-length: 20\r\n\
    \r\n\
    hello response! Lorem ipsum dolor";

    let mut rc = http::Collector::new();

    'collect: for chunk in sample.chunks(13) {
        let mut chunk = chunk;
        println!("[main] new chunk; {:?}", String::from_utf8_lossy(chunk));
        while !chunk.is_empty() {
            println!("[main] advance; chunk_window={:?}", String::from_utf8_lossy(chunk));
            let adv = rc.advance(chunk);

            println!("[main] got advance: {adv:?}");

            match adv.av_action {
                http::AvAction::Nop => {}
                other => {
                    print!("\x1b[95m");
                    match other {
                        http::AvAction::FirstLineReady(r) => {
                            println!("[main] first line ready: {:?}",
                                //http::first_line::FirstLine::from_bytes(&sample[r]));
                                String::from_utf8_lossy(&sample[r]));
                        }
                        http::AvAction::HeadersReady(r) => {
                            println!("[main] headers ready: {:?}",
                                String::from_utf8_lossy(&sample[r]));
                        }
                        http::AvAction::BodyReady(r) => {
                            println!("[main] body ready: {:?}",
                                String::from_utf8_lossy(&sample[r]));
                        }
                        http::AvAction::BodyChunkReady(r) => {
                            println!("[main] chunk ready: {:?}",
                                String::from_utf8_lossy(&sample[r]));
                        }
                        _ => unreachable!()
                    }
                    println!("\x1b[0m");
                }
            }

            if rc.finish_status().is_some() {
                println!("[main] finished: {:?}", rc.finish_status());
                break 'collect;
            }

            chunk = &chunk[adv.current..];

            println!("");
        }
        println!("");
    }
}

