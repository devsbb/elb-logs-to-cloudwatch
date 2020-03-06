use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{LogProcessor, RequestLogLine};

#[derive(Debug, Serialize, Deserialize)]
pub struct StdoutOutput {
    #[serde(skip, default = "crate::log_processing::csv_writer_builder")]
    writer: csv::WriterBuilder,
}

impl LogProcessor for StdoutOutput {
    fn process_line(&self, log_line: &RequestLogLine) -> Result<()> {
        self.writer
            .from_writer(std::io::stdout())
            .serialize(log_line)?;
        Ok(())
    }
}
