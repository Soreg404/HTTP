use crate::proto::consts::*;
use super::collect_error::CollectError;

pub struct FirstLineRequest {
    pub method: Method,
    pub target: Vec<u8>,
    pub version: Version
}
pub fn first_line_request(line: &[u8]) -> Result<FirstLineRequest, CollectError> {
        // example: GET / HTTP/1.1\r\n

        // ensure 3 parts (method, url and version)
        let mut line_splits = line.split(|c| *c == b' ');
        if line_splits.clone().count() != 3 {
            return Err(CollectError::TBD("expected 3 parts for first line".to_string()));
        }

        // take method
        let method_str = line_splits.next().unwrap();
        // todo: check if method bytes are valid
        let method = Method::from_bytes(method_str);

        // take url
        let url_str = line_splits.next().unwrap();
        let target = url_str.to_vec();

        // take version
        let version_bytes = line_splits.next().unwrap();
        let version = match Version::from_bytes(version_bytes) {
            Ok(v) => v,
            Err(()) => return Err(CollectError::TBD("take version".to_string()))
        };

        Ok(FirstLineRequest {
            method,
            target,
            version
        })
}
