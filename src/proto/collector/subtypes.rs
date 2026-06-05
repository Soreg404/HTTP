use super::collect_error::CollectError;
use crate::proto::consts::Version;

#[derive(Default)]
pub struct Request {

}

impl super::glue::Subtype for Request {
    fn first_line(
        &mut self,
        line: &[u8],
        version: &mut Version,
    ) -> Result<(), CollectError> {
        Err(CollectError::TBD(String::new()))
    }
}
