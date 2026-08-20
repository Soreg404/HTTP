use super::*;

pub struct HeadMeta {
    method: Range,
    target: Range,
    version: (),
    headers: Range,

    content_length: usize,
}

impl CHead {
    pub fn process(self, buffer: &[u8]) -> HeadMeta {
        let mut i = 0;

        let (method, target, r_version) = explode_first_line(
            strip_crlf(getline(&mut i, buffer))
        );
        // check validity of each


        let headers_start = i;
        let headers_end;
        let mut h_content_length = None;
        //let h_boundary = None;

        loop {
            debug_assert!(i < buffer.len());
            let line_base = i;
            let line = strip_crlf(getline(&mut i, buffer));
            if i == buffer.len() {
                headers_end = line_base;
                break;
            }

            let (r_h_name, r_h_body) = match get_header(line) {
                Ok(v) => v,
                Err(()) => unimplemented!("invalid header - missing ':'")
            };

            let h_name = &line[r_h_name];
            if h_name.eq_ignore_ascii_case(b"content-length") {
                if h_content_length.is_some() {
                    unimplemented!("duplicate content-length header");
                }
                let h_body = &line[r_h_body];
                let v = match bytes_to_usize(h_body) {
                    Ok(v) => v,
                    Err(()) => unimplemented!("invalid content-length value");
                };
                h_content_length = Some(v);
            }
            else if h_name.eq_ignore_ascii_case() {

            }

        }


        HeadMeta {
            method,
            target,
            version: (),
        }
    }
}

fn getline(i: &mut usize, buffer: &[u8]) -> &[u8] {
    let b = i;
    while *i < buffer.len() {
        if buffer[*i] == b'\n' {
            *i += 1;
            break;
        }
        *i += 1;
    }
    &buffer[b..i]
}

fn strip_crlf(line: &[u8]) -> &[u8] {
    match line.strip_suffix(b"\n") {
        Some(line) => line
            .strip_suffix(b"\r")
            .unwrap_or(line),
        None => line
    }
}

fn explode_first_line(buffer: &[u8]) -> Option<(Range, Range, Range)> {
    let mut i = 0;
    while i < buffer.len() && buffer[i] != b' ' {
        i += 1;
    }
    let method = 0..i;
    i += 1;

    let base = i;
    while i < buffer.len() && buffer[i] != b' ' {
        i += 1;
    }
    let target = base..i;
    i += 1;

    if method.len() == 0 || target.len() == 0 || i >= buffer.len(){
        return None;
    }

    let version = i..buffer.len();

    (method, target, version)
}

fn get_header(line: &[u8]) -> Result<(Range, Range)> {
    let mut i = 0;
    while i < line.len() && line[i] != b':' {
        i += 1;
    }
    if i == line.len() {
        return None;
    }
    let name = 0..i;
    i += 1;

    while i < line.len() && line[i].is_ascii_whitespace() {
        i += 1;
    }

    let mut j = line.len();
    while j > i && line[j - 1].is_ascii_whitespace() {
        j -= 1;
    }

    let body = i..j;

    Some((name, body))
}

fn bytes_to_usize(bytes: &[u8]) -> Result<usize, ()> {
    let mut i = 0;
    let mut base = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            return Err(());
        }
        base *= 10;
        base += bytes[i] - b'0';
        i += 1;
    }
    Ok(base)
}
