use crate::proto::consts::*;
use super::collect_error::CollectError;
use crate::proto::header_parser::HeaderBodyParser;
use index_slice::IndexSlice;

#[derive(Debug)]
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

pub struct ParsedContentDisposition {
    pub name: Option<IndexSlice>,
    pub filename: Option<IndexSlice>
}
pub fn parse_header_content_disposition(line: &[u8])
    -> Result<ParsedContentDisposition, CollectError> {
        let mut bp = HeaderBodyParser::new(line);
        if !bp.next_atom()
            .map(|v| v.as_slice_of(line).eq_ignore_ascii_case(b"attachment"))
                .unwrap_or(false) {
                    return Err(CollectError::TBD(
                            "idk invalid content-disposition; \
                        missing attachment atom".to_string()));
        }

        let mut name = None::<IndexSlice>;
        let mut filename = None::<IndexSlice>;

        match bp.next_attribute() {
            None | Some(Err(_)) => {
                return Err(CollectError::TBD("missing name".to_string()));
            }
            Some(Ok(v)) => {
                let k = v.key.as_slice_of(line);
                if k.eq_ignore_ascii_case(b"name") {
                    if name.is_some() {
                        return Err(CollectError::TBD("duplicate name".to_string()));
                    }
                    name = Some(v.value);
                } else if k.eq_ignore_ascii_case(b"filename") {
                    if filename.is_some() {
                        return Err(CollectError::TBD("duplicate filename".to_string()));
                    }
                    filename = Some(v.value);
                }
            }
        }

        match bp.next_attribute() {
            None => {}
            Some(Err(_)) => {
                return Err(CollectError::TBD("invalid content-disp".to_string()));
            }
            Some(Ok(v)) => {
                let k = v.key.as_slice_of(line);
                if k.eq_ignore_ascii_case(b"name") {
                    if name.is_some() {
                        return Err(CollectError::TBD("duplicate name".to_string()));
                    }
                    name = Some(v.value);
                } else if k.eq_ignore_ascii_case(b"filename") {
                    if filename.is_some() {
                        return Err(CollectError::TBD("duplicate filename".to_string()));
                    }
                    filename = Some(v.value);
                }
            }
        }

        Ok(ParsedContentDisposition {
            name,
            filename
        })
    }
