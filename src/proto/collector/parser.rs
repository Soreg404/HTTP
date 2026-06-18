use crate::proto::consts::*;
use super::collect_error::CollectError;

pub fn first_line_request(
    line: &[u8],
    method: &mut Method,
    url: &mut Vec<u8>,
    version: &mut Version
) -> Result<(), CollectError> {

}
