use anyhow::Error;
use serde::{Deserialize, Serialize};

use crate::types::{LogProcessor, RequestLogLine};

#[derive(Debug, Serialize, Deserialize)]
pub struct VoidOutput;
impl LogProcessor for VoidOutput {
    fn process_line(&self, _log_line: &RequestLogLine) -> Result<(), Error> {
        Ok(())
    }
}
