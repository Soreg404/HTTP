fn main() {
    let sample = b"\
        GET /target HTTP/1.1\r\n\
        header1: haha\r\n\
        transfer-encoding: chunked\r\n\
        \r\n\
        7\r\ncontent\r\n\
        f\r\ncontent content\r\n\
        1f\r\ncontent content content content\r\n\
        0\r\n";

    for i in 1..sample.len() {
        println!("\x1b[91m[main] chunk len: {i}\x1b[0m");

        let mut rc = http::Collector::new();

        'collect: for chunk in sample.chunks(i) {
            let mut chunk = chunk;
            //println!("[main] new chunk; {:?}", String::from_utf8_lossy(chunk));
            while !chunk.is_empty() {
                //println!("[main] advance; chunk_window={:?}", String::from_utf8_lossy(chunk));
                let adv = rc.advance(chunk);

                //println!("[main] got advance: {adv:?}");

                match adv.av_action {
                    http::AvAction::Nop => {}
                    other => {
                        print!("\x1b[95m");
                        match other {
                            http::AvAction::FirstLineReady(r) => {
                                println!("[main] first line ready: {:?}",
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

                //println!("");
            }
            //println!("");
        }
        if rc.finish_status().is_none() {
            panic!();
        }
    }
}
