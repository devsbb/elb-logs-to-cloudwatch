use std::io::Read;

use anyhow::{Context as _, Result};
use log::{info, trace};

use serde::de::DeserializeOwned;

use crate::pipelines::Pipeline;
use crate::types::RequestLogLine;

pub(crate) fn parse_log_stream<T, R>(file: R) -> impl Iterator<Item = Result<T>>
where
    T: DeserializeOwned,
    R: Read,
{
    csv::ReaderBuilder::new()
        .delimiter(b' ')
        .has_headers(false)
        .from_reader(file)
        .into_deserialize()
        .map(|f| {
            f.map_err(anyhow::Error::new)
                .context("failed to read a log line")
        })
}

pub(crate) fn process_log<R>(buffer: R, pipelines: &[(&Pipeline, wirefilter::Filter)]) -> Result<()>
where
    R: Read,
{
    let lines = parse_log_stream::<RequestLogLine, _>(buffer);
    info!("Processing file");

    for line in lines
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
            }
        }
    }

    info!("Processed");
    Ok(())
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
