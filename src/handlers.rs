use std::result::Result;

use aws_lambda_events::event::s3::S3Event;
use lambda_runtime::Context;
use log::{error, info};

use crate::config;

use crate::error::HandlerError;
use crate::log_processing::process_log;
use crate::pipelines::compile_pipelines;
use crate::s3::open_s3_file;

pub fn handler(event: S3Event, _context: Context) -> Result<(), HandlerError> {
    info!("Got an S3 event {:#?}", event);
    let config = config::from_args();
    let pipelines = compile_pipelines(&config);

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
        process_log(buffer, &pipelines).map_err(|error| {
            error!("Failed to process log file {:?}", error);
            HandlerError::Unknown(error)
        })?;
    }
    Ok(())
}
