use std::io::Read;

use anyhow::{Context as _, Result};
use log::{info, trace};

use serde::de::DeserializeOwned;

use crate::pipelines::Pipeline;
use crate::types::RequestLogLine;

const LOG_DELIMITER: u8 = b' ';

#[derive(Debug)]
pub(crate) struct ProcessLogOutput {
    pub total_lines: u64,
    pub matched_lines: u64,
}

pub(crate) fn parse_log_stream<T, R>(file: R) -> impl Iterator<Item = Result<T>>
where
    T: DeserializeOwned,
    R: Read,
{
    csv_reader_builder()
        .from_reader(file)
        .into_deserialize()
        .map(|f| {
            f.map_err(anyhow::Error::new)
                .context("failed to read a log line")
        })
}

pub(crate) fn process_log<R>(
    buffer: R,
    pipelines: &[(&Pipeline, wirefilter::Filter)],
) -> Result<ProcessLogOutput>
where
    R: Read,
{
    let mut total_lines = 0;
    let mut matched_lines = 0;
    let lines = parse_log_stream::<RequestLogLine, _>(buffer);
    info!("Processing file");

    for line in lines
        .inspect(|_| total_lines += 1)
        .filter(|line| {
            if line.is_ok() {
                return true;
            }
            trace!("Skipping line because of error {:?}", line);
            false
        })
        .map(Result::unwrap)
    {
        let context = line.execution_context()?;
        for (pipeline, filter) in pipelines {
            if filter.execute(&context).unwrap() {
                pipeline.output.get_log_processor().process_line(&line)?;
                matched_lines += 0;
            }
        }
    }

    info!("Processed");
    Ok(ProcessLogOutput {
        total_lines,
        matched_lines,
    })
}

pub(crate) fn csv_writer_builder() -> csv::WriterBuilder {
    let mut writer = csv::WriterBuilder::new();
    writer.delimiter(LOG_DELIMITER).has_headers(false);
    writer
}

pub(crate) fn csv_reader_builder() -> csv::ReaderBuilder {
    let mut reader = csv::ReaderBuilder::new();
    reader.delimiter(LOG_DELIMITER).has_headers(false);
    reader
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;

    use crate::log_processing::parse_log_stream;
    use crate::types::RequestLogLine;

    const GOOD_LOGS: &str = include_str!("../tests/fixtures/logs.txt");
    const BAD_LOGS: &str = include_str!("../tests/fixtures/bad_logs.txt");

    fn parse_logs(csv_data: &str) -> Vec<RequestLogLine> {
        let log_lines: Result<Vec<RequestLogLine>> =
            parse_log_stream(Cursor::new(csv_data)).collect();

        log_lines.unwrap()
    }

    #[test]
    fn test_log_parsing() {
        let log_lines = parse_logs(GOOD_LOGS);
        assert_eq!(log_lines.len(), 10);

        let bad_logs = parse_logs(BAD_LOGS);
        assert_eq!(bad_logs.len(), 13);
    }
}
