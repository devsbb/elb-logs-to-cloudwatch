use anyhow::{Context as _, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Type used to represent fields where AWS sends a `-` when the target group could not be reached
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MaybeNumber<T> {
    Number(T),
    FailedToParse(String),
}

#[derive(Debug, PartialEq)]
pub(crate) struct Request {
    pub method: String,
    pub path: String,
    pub http_version: String,
}

impl From<&str> for Request {
    fn from(data: &str) -> Self {
        let mut parts = data.split(" ");
        Request {
            method: parts.next().unwrap().to_owned(),
            path: parts.next().unwrap().to_owned(),
            http_version: parts.next().unwrap().to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RequestLogLine {
    pub request_type: String,
    pub timestamp: DateTime<Utc>,
    pub elb_name: String,
    pub client: String,
    pub target: String,
    pub request_processing_time: MaybeNumber<f64>,
    pub target_processing_time: MaybeNumber<f64>,
    pub response_processing_time: MaybeNumber<f64>,
    pub elb_status_code: u16,
    pub target_status_code: MaybeNumber<u16>,
    pub received_bytes: u64,
    pub sent_bytes: u64,
    // method + url + http version
    request: String,
    pub user_agent: String,
    pub ssl_cipher: String,
    pub ssl_protocol: String,
    pub target_group_arn: String,
    pub trace_id: String,
    pub domain_name: String,
    pub chosen_cert_arn: String,
    pub matched_rule_priority: String,
    pub request_creation_time: DateTime<Utc>,
    pub actions_executed: String,
    pub error_reason: String,
}

impl RequestLogLine {
    pub fn request(&self) -> Request {
        Request::from(self.request.as_str())
    }

    pub fn cloudwatch_dimension(&self) -> Result<CloudwatchDimension> {
        if self.target_group_arn == "-" {
            return Err(anyhow::anyhow!(format!(
                "invalid target group {}",
                self.target_group_arn
            )));
        }
        Ok(CloudwatchDimension {
            load_balancer: self.elb_name.clone(),
            target_group: self
                .target_group_arn
                .split(":")
                .last()
                .with_context(|| {
                    format!(
                        "failed to get a valid target from from {}",
                        self.target_group_arn
                    )
                })?
                .to_owned(),
        })
    }
}

#[derive(Debug)]
pub struct CloudwatchDimension {
    pub load_balancer: String,
    pub target_group: String,
}
