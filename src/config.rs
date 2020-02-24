use rusoto_core::Region;
use structopt::StructOpt;

use crate::pipelines::Pipelines;

#[derive(Debug, StructOpt)]
pub(crate) struct Config {
    #[structopt(short, long, env)]
    pub aws_region: Region,
    #[structopt(short, long, env)]
    pub pipelines: Pipelines,
    #[structopt(short, long, env)]
    pub bucket_name: String,
    pub bucket_keys: Vec<String>,
}

pub(crate) fn from_args() -> Config {
    Config::from_args()
}
