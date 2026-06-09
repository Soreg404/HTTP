use super::collect_error::CollectError;
use crate::proto::consts::Version;
use crate::proto::consts::Method;

#[derive(Default, Debug)]
pub struct Request {
    method: Method,
    url: String,
}

impl super::glue::Subtype for Request {
    fn first_line(
        &mut self,
        line: &[u8],
        version: &mut Version,
    ) -> Result<(), CollectError> {
        // example: GET / HTTP/1.1\r\n

        // ensure 3 parts (method, url and version)
        let mut line_splits = line.split(|c| *c == b' ');
        if line_splits.clone().count() != 3 {
            return Err(CollectError::TBD("expected 3 parts for first line".to_string()));
        }

        // take method
        let method_str = line_splits.next().unwrap();
        // todo: check if method bytes are valid
        self.method = crate::proto::consts::Method::from_bytes(method_str);

        // take url
        let url_str = line_splits.next().unwrap();
        self.url = String::from_utf8_lossy(url_str).to_string();

        // take version
        let version_bytes = line_splits.next().unwrap();
        *version = match Version::from_bytes(version_bytes) {
            Ok(v) => v,
            Err(()) => return Err(CollectError::TBD("take version".to_string()))
        };

        Ok(())
    }
}
