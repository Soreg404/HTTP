#[derive(Debug, Eq, PartialEq)]
enum CollectStage {
    CollectingHead,
    WaitingToProcessHead,
    CollectingBody,
    Done(Result<(), ()>)
}

#[derive(Debug)]
pub struct RequestCollector {
    stage: CollectStage,

    processed_bytes: usize,
    head_len: usize,
    body_len: usize,

    nl_cnt: u32,
}

pub struct ProcessBytes<'a> {
    pub state: ProcessBytesState,
    pub processed_bytes_total: usize,
    pub processed: &'a [u8],
    pub remainder: &'a [u8]
}
impl std::fmt::Debug for ProcessBytes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("ProcessBytes")
            .field("state", &self.state)
            .field("total", &self.processed_bytes_total)
            .field("processed", &String::from_utf8_lossy(self.processed))
            .field("remainder", &String::from_utf8_lossy(self.remainder))
            .finish()
    }
}

#[derive(Debug)]
pub enum ProcessBytesState {
    AllProcessed,
    CanProcessHead,
    Finished
}

impl RequestCollector {
    pub fn new() -> Self {
        Self {
            stage: CollectStage::CollectingHead,

            processed_bytes: 0,
            head_len: 0,
            body_len: 0,

            nl_cnt: 0
        }
    }
    pub fn get_total_processed_bytes(&self) -> usize {
        self.processed_bytes
    }
    pub fn get_head_len(&self) -> Option<usize> {
        match self.stage {
            CollectStage::CollectingHead | CollectStage::WaitingToProcessHead => None,
            _ => Some(self.body_len)
        }
    }
    pub fn get_body_len(&self) -> Option<usize> {
        match self.stage {
            CollectStage::CollectingHead => None,
            _ => Some(self.head_len)
        }
    }
    pub fn process_bytes<'a>(&mut self, bytes: &'a [u8]) -> ProcessBytes<'a> {
        match self.stage {
            CollectStage::CollectingHead => {
                let mut i = 0;
                while i < bytes.len() {
                    if !bytes[i].is_ascii() {
                        self.stage = CollectStage::Done(Err(()));
                        let (processed, remainder) = bytes.split_at(i);
                        return ProcessBytes {
                            state: ProcessBytesState::Finished,
                            processed_bytes_total: self.processed_bytes,
                            processed,
                            remainder
                        }
                    }

                    self.processed_bytes += 1;
                    if bytes[i] == b'\n' {
                        self.nl_cnt += 1;
                    } else if bytes[i] != b'\r' {
                        self.nl_cnt = 0;
                    }
                    i += 1;

                    if self.nl_cnt == 2 {
                        self.stage = CollectStage::WaitingToProcessHead;
                        self.head_len = self.processed_bytes;
                        let (processed, remainder) = bytes.split_at(i);
                        return ProcessBytes {
                            state: ProcessBytesState::CanProcessHead,
                            processed_bytes_total: self.processed_bytes,
                            processed,
                            remainder
                        }
                    }
                }
                ProcessBytes {
                    state: ProcessBytesState::AllProcessed,
                    processed_bytes_total: self.processed_bytes,
                    processed: bytes,
                    remainder: &[]
                }
            }
            CollectStage::WaitingToProcessHead => {
                ProcessBytes {
                    state: ProcessBytesState::CanProcessHead,
                    processed_bytes_total: self.processed_bytes,
                    processed: &[],
                    remainder: bytes
                }
            }
            CollectStage::CollectingBody => {
                let current = self.processed_bytes + bytes.len();
                let all = self.head_len + self.body_len;
                if current >= all {
                    let (processed, remainder) = bytes.split_at(bytes.len() - (current - all));
                    self.processed_bytes += processed.len();
                    self.stage = CollectStage::Done(Ok(()));

                    ProcessBytes {
                        state: ProcessBytesState::Finished,
                        processed_bytes_total: self.processed_bytes,
                        processed,
                        remainder
                    }
                }
                else {
                    self.processed_bytes += bytes.len();
                    ProcessBytes {
                        state: ProcessBytesState::AllProcessed,
                        processed_bytes_total: self.processed_bytes,
                        processed: bytes,
                        remainder: &[]
                    }
                }
            }
            CollectStage::Done(_r) => {
                ProcessBytes {
                    state: ProcessBytesState::Finished,
                    processed_bytes_total: self.processed_bytes,
                    processed: &[],
                    remainder: bytes
                }
            }
        }
    }

    pub fn process_head(&mut self, buffer: &[u8]) {
        assert!(buffer.len() >= self.head_len);
        assert!(self.stage == CollectStage::WaitingToProcessHead);

        let mut buffer = &buffer[..self.head_len];

        enum HeadStage {
            FirstLine,
            Headers
        }
        let mut head_stage = HeadStage::FirstLine;

        loop {
            let line = match take_line(buffer) {
                None => {
                    panic!("missing next line");
                }
                Some(v) => {
                    buffer = v.rest;
                    v.line
                }
            };

            match head_stage {
                HeadStage::FirstLine => {
                    let mut parts = line.split(|c| *c == b' ');
                    if parts.clone().count() != 3 {
                        panic!("invalid 1st line");
                    }
                    let method = parts.next().unwrap();
                    let target = parts.next().unwrap();
                    let version = parts.next().unwrap();
                    println!(
                        "method={:?}, target={:?}, version={:?}",
                        String::from_utf8_lossy(method),
                        String::from_utf8_lossy(target),
                        String::from_utf8_lossy(version),
                    );
                    head_stage = HeadStage::Headers;
                }
                HeadStage::Headers => {
                    if line.is_empty() {
                        println!("done");
                        break;
                    }

                    let ci = line.iter().position(|c| *c == b':');
                    let ci = match ci {
                        None => {
                            panic!("invalid header: {:?}",
                                String::from_utf8_lossy(line));
                        }
                        Some(v) => v
                    };
                    let (fld_name, fld_body) = line.split_at(ci);
                    let fld_body = &fld_body[1..];
                    println!();
                    println!(
                        "fld_name={:?}, fld_body={:?}",
                        String::from_utf8_lossy(fld_name),
                        String::from_utf8_lossy(fld_body),
                    );
                }
            }
        }

        // eventually
        self.stage = CollectStage::CollectingBody;
    }
}

#[derive(Debug, Eq, PartialEq)]
struct TakeLine<'a> {
    line: &'a [u8],
    crlf: &'a [u8],
    rest: &'a [u8]
}
fn take_line<'a>(buffer: &'a [u8]) -> Option<TakeLine<'a>> {
    let mut i = 0;
    while i < buffer.len() && buffer[i] != b'\n' {
        i += 1;
    }
    if i == buffer.len() {
        return None;
    }
    let (line_crlf, rest) = buffer.split_at(i + 1);
    let (line, crlf) = strip_crlf(line_crlf)
        .unwrap();
    Some(TakeLine {line, crlf, rest})
}
#[test]
fn test_take_line() {
    let buffer = b"line one\r\nline two\ninvalid";
    let r = take_line(buffer).unwrap();
    assert_eq!(r.line, b"line one");
    assert_eq!(r.crlf, b"\r\n");
    assert_eq!(r.rest, &buffer[10..]);
    let buffer = r.rest;

    let r = take_line(&buffer).unwrap();
    assert_eq!(r.line, b"line two");
    assert_eq!(r.crlf, b"\n");
    assert_eq!(r.rest, &buffer[9..]);
    let buffer = r.rest;

    assert_eq!(take_line(&buffer), None);
}

pub fn strip_crlf(buffer: &[u8]) -> Option<(&[u8], &[u8])> {
    let stripped = match buffer.last() {
        Some(b) if *b == b'\n' => {
            &buffer[..buffer.len() - 1]
        }
        _ => return None
    };
    let stripped = match stripped.last() {
        Some(b) if *b == b'\r' => {
            &stripped[..stripped.len() - 1]
        }
        _ => stripped
    };
    Some((stripped, &buffer[stripped.len()..]))
}
#[test]
fn test_strip_crlf() {
    assert_eq!(strip_crlf(b"hello\r\n"), Some((b"hello" as _, b"\r\n" as _)));
    assert_eq!(strip_crlf(b"hello\n"), Some((b"hello" as _, b"\n" as _)));
    assert_eq!(strip_crlf(b"hello"), None);
}
