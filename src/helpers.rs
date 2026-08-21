macro_rules! trace {
    ($($arg:tt)*) => {
        #[cfg(any(test, trace))]
        println!("\x1b[96mtrace!\x1b[0m [{}:{:04}] {}",
            file!(),
            line!(),
            format_args!($($arg)*));
    };
}
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        #[cfg(any(test))]
        println!("\x1b[93mdebug_trace!\x1b[0m [{}:{:04}] {}",
            file!(),
            line!(),
            format_args!($($arg)*));
    };
}

pub fn split_crlf(s: &[u8]) -> (&[u8], &[u8]) {
    let mut sp = s.len();
    match s.last() {
        Some(b'\n') => { sp -= 1; }
        _ => return (s, &[])
    }
    match &s[..sp].last() {
        Some(b'\r') => { sp -= 1; }
        _ => {}
    }
    s.split_at(sp)
}
#[test]
fn test_split_crlf() {
    assert_eq!(split_crlf(b"hello\r\n"), (b"hello".as_slice(), b"\r\n".as_slice()));
    assert_eq!(split_crlf(b"hello\n\r"), (b"hello\n\r".as_slice(), b"".as_slice()));
    assert_eq!(split_crlf(b"hello\n"),   (b"hello".as_slice(), b"\n".as_slice()));
}

pub fn usize_from_u8_slice(s: &[u8]) -> Option<usize> {
    usize_from_u8_slice_(s, 10, digit)
}

pub fn usize_from_u8_slice_hex(s: &[u8]) -> Option<usize> {
    usize_from_u8_slice_(s, 16, hexit)
}

fn usize_from_u8_slice_<F>(
    s: &[u8],
    multiplier: usize,
    func: F
) -> Option<usize> 
where F: Fn(u8) -> Option<u8> {
    let mut n = 0;
    let mut i = 0;
    while i < s.len() {
        let d = func(s[i])? as usize;
        n *= multiplier;
        n += d;
        i += 1;
    }
    Some(n)
}

pub fn digit(c: u8) -> Option<u8> {
    Some(match c {
        b'0'..=b'9' => c - b'0',
        _ => return None
    })
}

pub fn hexit(c: u8) -> Option<u8> {
    Some(match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => return None
    })
}
