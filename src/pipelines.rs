use std::str::FromStr;

use serde::{Deserialize, Serialize};
use wirefilter::Scheme;

use crate::config::Config;
use crate::output::OutputType;

lazy_static::lazy_static! {
    pub(crate) static ref SCHEME: Scheme = Scheme! {
        elb_status_code: Int,
        user_agent: Bytes,
        target_group_arn: Bytes,
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Pipeline {
    pub filter: String,
    pub output: OutputType,
}

impl Pipeline {
    pub fn get_filter(&self) -> wirefilter::Filter {
        let ast = SCHEME
            .parse(self.filter.as_str())
            .unwrap_or_else(|_| panic!("Failed to parse the input filter: {:?}", self.filter));
        ast.compile()
    }
}

#[derive(Debug)]
pub(crate) struct Pipelines(Vec<Pipeline>);

impl Pipelines {
    pub fn inner(&self) -> &Vec<Pipeline> {
        &self.0
    }
}

impl FromStr for Pipelines {
    type Err = anyhow::Error;

    fn from_str(json: &str) -> Result<Self, Self::Err> {
        Ok(Self(serde_json::from_str(json)?))
    }
}

pub(crate) fn compile_pipelines(config: &Config) -> Vec<(&Pipeline, wirefilter::Filter)> {
    config
        .pipelines
        .inner()
        .iter()
        .map(|pipeline| (pipeline, pipeline.get_filter()))
        .collect()
}
