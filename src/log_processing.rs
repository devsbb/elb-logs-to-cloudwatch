use std::collections::HashSet;
use std::io::Read;

use anyhow::{Context as _, Result};
use itertools::Itertools;
use log::{info, trace};
use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, Dimension, MetricDatum, PutMetricDataInput};
use serde::de::DeserializeOwned;

use crate::s3::process_s3_file;
use crate::types::RequestLogLine;
use crate::CONFIG;

const CLOUDWATCH_BATCH_SIZE: usize = 20;

lazy_static::lazy_static! {
    static ref INTERESTING_ERRORS: HashSet<u16> = {
        let mut hash = HashSet::new();
        for code in [502, 503].iter() {
            hash.insert(*code);
        }
        hash
    };
}

thread_local! {
    static CLOUDWATCH_CLIENT: CloudWatchClient = CloudWatchClient::new(CONFIG.aws_region.clone());
}

pub(crate) fn read_log_file<T, R>(file: R) -> impl Iterator<Item = Result<T>>
where
    T: DeserializeOwned,
    R: Read,
{
    csv::ReaderBuilder::new()
        .delimiter(b' ')
        .has_headers(false)
        .from_reader(file)
        .into_deserialize()
        .map(|f| f.map_err(|e| anyhow::Error::new(e).context("failed to read a log line")))
}

pub(crate) fn process_log_file(bucket: &str, key: &str) -> Result<()> {
    let buffer = process_s3_file(bucket, key)?;
    let lines = read_log_file::<RequestLogLine, _>(buffer);
    info!("Processing {}", key);
    for lines_result in lines
        .filter(|line| {
            if line.is_ok() {
                return true;
            }
            trace!("Skipping line because of error {:?}", line);
            return false;
        })
        .map(Result::unwrap)
        .filter(|line| INTERESTING_ERRORS.contains(&line.elb_status_code))
        .chunks(CLOUDWATCH_BATCH_SIZE)
        .into_iter()
        .map(|lines| process_log_line(lines.collect()))
    {
        info!("Processed {:?} lines", lines_result?);
    }

    info!("Processed {}", key);
    Ok(())
}

fn process_log_line(lines: Vec<RequestLogLine>) -> Result<usize> {
    let input = PutMetricDataInput {
        namespace: CONFIG.cloudwatch_namespace.clone(),
        metric_data: lines
            .iter()
            .map(log_line_to_metric)
            .collect::<Result<Vec<MetricDatum>>>()
            .context("error converting log line to metric")?,
    };

    let response =
        CLOUDWATCH_CLIENT.with(|client: &CloudWatchClient| client.put_metric_data(input).sync());
    response
        .map_err(|e| {
            let context = format!("error sending metric {:?}", e);
            anyhow::Error::new(e).context(context)
        })
        .unwrap();
    Ok(lines.len())
}

fn log_line_to_metric(line: &RequestLogLine) -> Result<MetricDatum> {
    let dimension = line.cloudwatch_dimension()?;
    Ok(MetricDatum {
        dimensions: Some(vec![
            Dimension {
                name: "TargetGroup".to_string(),
                value: dimension.target_group,
            },
            Dimension {
                name: "LoadBalancer".to_string(),
                value: dimension.load_balancer,
            },
        ]),
        metric_name: CONFIG.cloudwatch_metric_name.clone(),
        value: Some(1.0),
        unit: Some("Count".to_string()),
        timestamp: Some(line.request_creation_time.to_rfc3339()),
        ..Default::default()
    })
}
