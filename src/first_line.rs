use crate::defs::*;

pub struct FirstLine<'a> {
    pub method: MethodRaw<'a>,
    pub target_str: &'a [u8],
    pub version: (u8, u8),
}

impl<'a> FirstLine<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ()> {
        let bytes = crate::split_crlf(bytes).0;
        let mut i = 0;
        while i < bytes.len() && bytes[i] != b' ' {
            i += 1;
        }
        let method_str = &bytes[..i];
        let method = MethodRaw::from_bytes(method_str);

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

        if bytes.len() - i != 8 {
            return Err(());
        }
        let ver_str = &bytes[i + 1 ..];
        if !ver_str.starts_with(b"HTTP/") || ver_str[6] != b'.'
            || !ver_str[5].is_ascii_digit() || !ver_str[7].is_ascii_digit() {
                return Err(());
        }
        let hi = ver_str[5] - b'0';
        let lo = ver_str[7] - b'0';
        let version = (hi, lo);

        Ok(Self {
            method,
            target_str,
            version
        })
    }
}
