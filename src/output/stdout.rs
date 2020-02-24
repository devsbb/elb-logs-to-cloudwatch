use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{LogProcessor, RequestLogLine};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StdoutOutput;
impl LogProcessor for StdoutOutput {
    fn process_line(&self, log_line: &RequestLogLine) -> Result<()> {
        println!("{:?}", log_line);
        Ok(())
    }
}
