use std::cell::RefCell;

use anyhow::{Context, Result};
use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, Dimension, MetricDatum, PutMetricDataInput};
use rusoto_core::Region;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::output::buffered_trait::BufferedLogProcessor;
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
        self.add_to_queue(&log_line)?;
        Ok(())
    }
}
impl BufferedLogProcessor for CloudwatchMetricOutput {
    fn maximum_buffer_size(&self) -> usize {
        CLOUDWATCH_BATCH_SIZE
    }

    fn buffer_len(&self) -> usize {
        self.buffer.borrow().len()
    }

    fn buffer_clear(&self) {
        self.buffer.borrow_mut().clear();
    }

    fn push_to_buffer(&self, log_line: RequestLogLine) {
        self.buffer.borrow_mut().push(log_line);
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
}

impl CloudwatchMetricOutput {
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
