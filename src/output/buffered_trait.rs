use anyhow::Result;
use log::debug;

use crate::types::RequestLogLine;

pub(crate) trait BufferedLogProcessor {
    fn maximum_buffer_size(&self) -> usize;
    fn buffer_len(&self) -> usize;
    fn buffer_clear(&self);
    fn push_to_buffer(&self, log_line: RequestLogLine);
    fn process_log_lines(&self) -> Result<()>;

    fn add_to_queue(&self, log_line: &RequestLogLine) -> Result<()> {
        if self.is_full() {
            self.flush()?;
        }

        self.push_to_buffer((*log_line).clone());

        Ok(())
    }

    fn flush(&self) -> Result<()> {
        debug!("Flushing {} items", self.buffer_len());
        if self.buffer_is_empty() {
            return Ok(());
        }
        self.process_log_lines()?;
        self.buffer_clear();
        Ok(())
    }

    fn is_full(&self) -> bool {
        self.buffer_len() == self.maximum_buffer_size()
    }

    fn buffer_is_empty(&self) -> bool {
        self.buffer_len() == 0
    }
}
