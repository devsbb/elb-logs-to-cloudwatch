use std::result::Result;

use aws_lambda_events::event::s3::S3Event;
use lambda_runtime::Context;
use log::{error, info, trace};

use crate::config;

use crate::error::HandlerError;
use crate::log_processing::process_log;
use crate::pipelines::compile_pipelines;
use crate::s3::open_s3_file;
use std::time::Instant;

pub fn handler(event: S3Event, _context: Context) -> Result<(), HandlerError> {
    trace!("Got an S3 event {:#?}", event);
    let config = config::from_args();
    let pipelines = compile_pipelines(&config.pipelines);
    let start_time = Instant::now();
    let mut total_lines = 0;
    let mut matched_lines = 0;

    for record in event.records {
        let buffer = open_s3_file(
            &record.s3.bucket.name.unwrap(),
            &record.s3.object.key.unwrap(),
            &config,
        )
        .map_err(|error| {
            error!("Failed to read S3 file {:?}", error);
            HandlerError::S3Error(error)
        })?;
        let output = process_log(buffer, &pipelines).map_err(|error| {
            error!("Failed to process log file {:?}", error);
            HandlerError::Unknown(error)
        })?;
        total_lines += output.total_lines;
        matched_lines += output.matched_lines;
    }
    let end_time = start_time.elapsed();
    info!(
        "Finished processing {} lines with {} matches in {:?}",
        total_lines, matched_lines, end_time
    );
    Ok(())
}
