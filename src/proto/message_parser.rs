use crate::proto::consts::*;
use super::collect_error::CollectError;

pub struct FirstLineRequest {
    method: Method,
    target: Vec<u8>,
    version: Version
}
pub fn first_line_request(
    line: &[u8],
) -> Result<FirstLineRequest, CollectError> {

}
