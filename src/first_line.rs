use crate::defs::*;
use crate::helpers::*;

#[derive(Debug, Eq, PartialEq)]
pub struct FirstLineRequest<'a> {
    pub method: MethodRaw<'a>,
    pub target_str: &'a [u8],
    pub version: (u8, u8),
}

impl<'a> FirstLineRequest<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ()> {
        debug_trace!("first line start; bytes({})={:?}", bytes.len(), String::from_utf8_lossy(bytes));
        let bytes = split_crlf(bytes).0;
        debug_trace!("bytes no crlf: ({})={:?}", bytes.len(), String::from_utf8_lossy(bytes));
        let mut i = 0;
        while i < bytes.len() && bytes[i] != b' ' {
            i += 1;
        }
        let method_str = &bytes[..i];
        let method = MethodRaw::from_bytes(method_str);
        debug_trace!("method={method:?}; i={i}; rest={:?}", String::from_utf8_lossy(&bytes[i..]));

        i += 1;
        if i >= bytes.len() {
            return Err(());
        }

        let target_start = i;
        while i < bytes.len() && bytes[i] != b' ' {
            if !bytes[i].is_ascii_graphic() {
                return Err(());
            }
            i += 1;
        }
        let target_str = &bytes[target_start..i];
        if i == bytes.len() {
            return Err(());
        }
        i += 1;

        debug_trace!("target_str={:?}; rest={:?}",
            String::from_utf8_lossy(target_str),
            String::from_utf8_lossy(&bytes[i..]),
        );

        if bytes.len() - i != 8 {
            return Err(());
        }
        let ver_str = &bytes[i..];
        if !ver_str.starts_with(b"HTTP/") || ver_str[6] != b'.'
            || !ver_str[5].is_ascii_digit() || !ver_str[7].is_ascii_digit() {
                return Err(());
        }
        let hi = ver_str[5] - b'0';
        let lo = ver_str[7] - b'0';
        let version = (hi, lo);
        debug_trace!("version={version:?}");

        Ok(Self {
            method,
            target_str,
            version
        })
    }
}

#[test]
fn test_first_line_request() {
    assert_eq!(
        FirstLineRequest::from_bytes(b"GET / HTTP/1.1\r\n"),
        Ok(FirstLineRequest {
            method: MethodRaw::Known(Method::Get),
            target_str: b"/",
            version: (1, 1)
        })
    );
}
