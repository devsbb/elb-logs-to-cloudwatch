use std::result::Result;

use aws_lambda_events::event::s3::S3Event;
use lambda_runtime::Context;
use log::{error, info};

use crate::error::HandlerError;
use crate::log_processing::process_log_file;

pub fn handler(event: S3Event, _context: Context) -> Result<(), HandlerError> {
    info!("Got an S3 event {:#?}", event);
    for record in event.records {
        process_log_file(
            &record.s3.bucket.name.unwrap(),
            &record.s3.object.key.unwrap(),
        )
        .map_err(|error| {
            error!("Failed to process log file {:?}", error);
            HandlerError::Unknown(error)
        })?;
    }
    Ok(())
}
