use std::cell::RefCell;
use std::io::{Cursor, Read};

use anyhow::Result;
use itertools::Itertools;
use log::{error, trace};
use rusoto_core::Region;
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogStreamsRequest, InputLogEvent,
    PutLogEventsRequest,
};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::log_processing::csv_writer_builder;
use crate::output::buffered_trait::BufferedLogProcessor;
use crate::types::{LogProcessor, RequestLogLine};

const BUFFER_SIZE: usize = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudwatchLogOutput {
    pub group_name: String,
    pub stream_name: String,
    #[serde(skip)]
    buffer: RefCell<SmallVec<[RequestLogLine; BUFFER_SIZE]>>,
    #[serde(skip)]
    sequence_token: RefCell<Option<String>>,
    #[serde(skip)]
    aws_region: Region,
}

impl LogProcessor for CloudwatchLogOutput {
    fn process_line(&self, log_line: &RequestLogLine) -> Result<()> {
        self.add_to_queue(&log_line)?;
        Ok(())
    }
}

impl BufferedLogProcessor for CloudwatchLogOutput {
    fn maximum_buffer_size(&self) -> usize {
        BUFFER_SIZE
    }

    fn buffer_len(&self) -> usize {
        self.buffer.borrow().len()
    }

    fn buffer_clear(&self) {
        self.buffer.borrow_mut().clear()
    }

    fn push_to_buffer(&self, log_line: RequestLogLine) {
        self.buffer.borrow_mut().push(log_line);
    }

    fn process_log_lines(&self) -> Result<()> {
        let cli = CloudWatchLogsClient::new(self.aws_region.clone());
        self.fetch_next_sequence_token(&cli)?;

        let request = PutLogEventsRequest {
            log_events: self
                .buffer
                .borrow()
                .iter()
                .sorted_by_key(|line| line.timestamp)
                .map(|line| self.process_log_line(line))
                .collect::<Result<_>>()?,
            log_group_name: self.group_name.clone(),
            log_stream_name: self.stream_name.clone(),
            sequence_token: self.sequence_token.borrow().clone(),
        };

        let response = cli.put_log_events(request).sync()?;

        *self.sequence_token.borrow_mut() = response.next_sequence_token;

        if let Some(rejected_events) = response.rejected_log_events_info {
            error!(
                "There were some rejected events (indexes) expired: {:?} too new: {:?} too old: {:?}",
                rejected_events.expired_log_event_end_index,
                rejected_events.too_new_log_event_start_index,
                rejected_events.too_old_log_event_end_index
            );
        }

        Ok(())
    }
}

impl CloudwatchLogOutput {
    fn process_log_line(&self, line: &RequestLogLine) -> Result<InputLogEvent> {
        let mut buffer = Cursor::new(Vec::new());
        csv_writer_builder()
            .from_writer(buffer.by_ref())
            .serialize(line)?;

        Ok(InputLogEvent {
            message: String::from_utf8_lossy(buffer.get_ref()).to_string(),
            timestamp: line.timestamp.timestamp_millis(),
        })
    }

    fn fetch_next_sequence_token(&self, cli: &CloudWatchLogsClient) -> Result<()> {
        if self.sequence_token.borrow().is_some() {
            return Ok(());
        }
        trace!("Trying to fetch next sequence token");

        let request = DescribeLogStreamsRequest {
            limit: Some(1),
            log_group_name: self.group_name.clone(),
            log_stream_name_prefix: Some(self.stream_name.clone()),
            ..Default::default()
        };
        let response = cli.describe_log_streams(request).sync()?;
        if let Some(streams) = response.log_streams {
            if let Some(stream) = streams.get(0) {
                trace!(
                    "Fetched next token {:?} response {:#?}",
                    stream.upload_sequence_token,
                    stream,
                );
                *self.sequence_token.borrow_mut() = stream.upload_sequence_token.clone()
            } else {
                trace!("Streams were empty");
            }
        } else {
            trace!("No streams found");
        };

        Ok(())
    }
}
