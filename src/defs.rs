#[derive(Debug, Eq, PartialEq)]
pub enum MethodRaw<'a> {
    Known(Method),
    Unknown(&'a [u8])
}

#[derive(Debug, Eq, PartialEq)]
pub enum Method {
    Get,
    Post
}
impl<'a> MethodRaw<'a> {
    pub fn from_bytes(v: &'a [u8]) -> Self {
        match v {
            b"GET" => Self::Known(Method::Get),
            b"POST" => Self::Known(Method::Post),
            v => Self::Unknown(v)
        }
    }
}

