use serde::{Deserialize, Serialize};

pub use crate::output::cloudwatch_logs::CloudwatchLogOutput;
pub use crate::output::cloudwatch_metric::CloudwatchMetricOutput;
pub use crate::output::stdout::StdoutOutput;
pub use crate::output::void::VoidOutput;
use crate::types::LogProcessor;

pub mod buffered_trait;
pub mod cloudwatch_logs;
pub mod cloudwatch_metric;
pub mod stdout;
pub mod void;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OutputType {
    #[serde(rename = "cloudwatch_metric")]
    CloudwatchMetric(CloudwatchMetricOutput),
    #[serde(rename = "cloudwatch_log")]
    CloudwatchLog(CloudwatchLogOutput),
    #[serde(rename = "stdout")]
    Stdout(StdoutOutput),
    #[serde(rename = "void")]
    Void(VoidOutput),
}

impl OutputType {
    pub fn get_log_processor(&self) -> &dyn LogProcessor {
        match self {
            OutputType::CloudwatchMetric(o) => o,
            OutputType::CloudwatchLog(o) => o,
            OutputType::Stdout(o) => o,
            OutputType::Void(o) => o,
        }
    }
}
