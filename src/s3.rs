use anyhow::{Context as _, Result};
use flate2::read::MultiGzDecoder;
use log::info;
use rusoto_s3::{S3Client, S3};
use serde::de::DeserializeOwned;

use crate::log_processing::read_log_file;
use crate::CONFIG;

lazy_static::lazy_static! {
    static ref S3_CLIENT: S3Client = S3Client::new(CONFIG.aws_region.clone());
}

pub(crate) fn process_s3_file<T>(bucket: &str, key: &str) -> Result<impl Iterator<Item = Result<T>>>
where
    T: DeserializeOwned,
{
    info!("Starting to download from s3://{}/{}", bucket, key);
    let request = rusoto_s3::GetObjectRequest {
        bucket: bucket.to_owned(),
        key: key.to_owned(),
        ..Default::default()
    };
    let response = S3_CLIENT.get_object(request).sync()?;

    let body = response.body.context("No body found for this key")?;

    Ok(read_log_file(MultiGzDecoder::new(
        body.into_blocking_read(),
    )))
}
