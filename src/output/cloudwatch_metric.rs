use std::cell::RefCell;

use anyhow::{Context, Result};
use log::{debug, trace};
use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, Dimension, MetricDatum, PutMetricDataInput};
use rusoto_core::Region;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::types::{LogProcessor, RequestLogLine};

const CLOUDWATCH_BATCH_SIZE: usize = 20;

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudwatchMetricOutput {
    pub namespace: String,
    pub metric_name: String,
    #[serde(skip)]
    buffer: RefCell<SmallVec<[RequestLogLine; CLOUDWATCH_BATCH_SIZE]>>,
    #[serde(skip)]
    aws_region: Region,
}

#[derive(Debug)]
pub struct CloudwatchDimension {
    pub load_balancer: String,
    pub target_group: String,
}

impl LogProcessor for CloudwatchMetricOutput {
    fn process_line(&self, log_line: &RequestLogLine) -> Result<()> {
        self.push(&log_line)?;
        Ok(())
    }
}

impl CloudwatchMetricOutput {
    fn push(&self, log_line: &RequestLogLine) -> Result<()> {
        if self.is_full() {
            self.flush()?;
        }

        if log::log_enabled!(log::Level::Trace) {
            trace!("Adding item, previous count {}", self.buffer.borrow().len());
        }
        self.buffer.borrow_mut().push((*log_line).clone());

        Ok(())
    }

    fn flush(&self) -> Result<()> {
        debug!("Flushing {} items", self.buffer.borrow().len());
        if self.buffer.borrow().is_empty() {
            return Ok(());
        }
        self.process_log_lines()?;
        self.buffer.borrow_mut().clear();
        Ok(())
    }

    fn is_full(&self) -> bool {
        self.buffer.borrow().len() == CLOUDWATCH_BATCH_SIZE
    }

    fn process_log_lines(&self) -> Result<()> {
        let input = PutMetricDataInput {
            namespace: self.namespace.clone(),
            metric_data: self
                .buffer
                .borrow()
                .iter()
                .map(|e| self.log_line_to_metric(e))
                .collect::<Result<Vec<MetricDatum>>>()
                .context("error converting log line to metric")?,
        };

        let client = CloudWatchClient::new(self.aws_region.clone());
        let response = client.put_metric_data(input).sync();
        response
            .map_err(|e| {
                let context = format!("error sending metric {:?}", e);
                anyhow::Error::new(e).context(context)
            })
            .unwrap();
        Ok(())
    }

    fn log_line_to_metric(&self, line: &RequestLogLine) -> Result<MetricDatum> {
        let dimension = self.cloudwatch_dimension(line)?;
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
            metric_name: self.metric_name.clone(),
            value: Some(1.0),
            unit: Some("Count".to_string()),
            timestamp: Some(line.request_creation_time.to_rfc3339()),
            ..Default::default()
        })
    }

    fn cloudwatch_dimension(&self, line: &RequestLogLine) -> Result<CloudwatchDimension> {
        if line.target_group_arn == "-" {
            return Err(anyhow::anyhow!(format!(
                "invalid target group {}",
                line.target_group_arn
            )));
        }
        Ok(CloudwatchDimension {
            load_balancer: line.elb_name.clone(),
            target_group: line
                .target_group_arn
                .split(':')
                .last()
                .with_context(|| {
                    format!(
                        "failed to get a valid target from from {}",
                        line.target_group_arn
                    )
                })?
                .to_owned(),
        })
    }
}

impl Drop for CloudwatchMetricOutput {
    fn drop(&mut self) {
        self.flush().expect("failed to flush metrics");
    }
}
