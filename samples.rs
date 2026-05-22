/**
* Samples could be a directory.
* Importing a sample could be done with the include_bytes! macro
**/

pub const MULTIPART1: &[u8] = b"\
GET /resource/entity/flying%20bison HTTP/1.1\r\n\
host: localhost\r\n\
content-length: 112\r\n\
content-type: multipart/form-data; boundary=abc\r\n\
\r\n\
skip text\r\n\
--abc\r\n\
content-disposition: form-data; filename=\"abcd.jpg\"; name=\"tf\"\r\n\
content-type: text/plain\r\n\
\r\n\
hello worl!!\r\n\
--abc--\r\n\
";

pub const MINIMAL: &[u8] = b"GET /path/?query=val#frag HTTP/1.1\r\n\r\n";
pub const MINIMAL2: &[u8] = b"GET /?# HTTP/1.1\r\n\r\n";
pub const MINIMAL3: &[u8] = b"GET / HTTP/1.1\r\n\r\n";
