use crate::output::cloudwatch_metric::CloudwatchMetricOutput;
use crate::output::stdout::StdoutOutput;
use crate::types::LogProcessor;

pub mod cloudwatch_metric;
pub mod stdout;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum OutputType {
    #[serde(rename = "cloudwatch_metric")]
    CloudwatchMetric(CloudwatchMetricOutput),
    #[serde(rename = "stdout")]
    Stdout(StdoutOutput),
}

impl OutputType {
    pub fn get_log_processor(&self) -> &dyn LogProcessor {
        match self {
            OutputType::CloudwatchMetric(o) => o,
            OutputType::Stdout(o) => o,
        }
    }
}
