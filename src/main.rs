use std::env;
use std::fmt::Debug;

use anyhow::Result;
use env_logger::DEFAULT_FILTER_ENV;
use lambda_runtime::lambda;
use log::info;
use rusoto_core::Region;
use structopt::StructOpt;

use crate::handlers::handler;
use crate::log_processing::process_log_file;
use std::env::var_os;

mod error;
mod handlers;
mod log_processing;
mod s3;
mod types;

const THREADS_NUMBER: usize = 10;
lazy_static::lazy_static! {
    pub(crate) static ref CONFIG: Config = Config::from_args();
}

#[derive(Debug, StructOpt)]
struct Config {
    #[structopt(short, long, env)]
    pub aws_region: Region,
    #[structopt(short = "n", long, env)]
    pub cloudwatch_namespace: String,
    #[structopt(short = "m", long, env)]
    pub cloudwatch_metric_name: String,
    #[structopt(short, long, env)]
    pub bucket_name: String,
    pub bucket_keys: Vec<String>,
}

fn main() -> Result<()> {
    if env::var_os(DEFAULT_FILTER_ENV).is_none() {
        let log_level = format!("{}=info", env!("CARGO_PKG_NAME").replace("-", "_"));
        env::set_var(DEFAULT_FILTER_ENV, &log_level);
        env_logger::init();
        info!("Setting log level to {}", log_level);
    } else {
        env_logger::init();
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(THREADS_NUMBER)
        .build_global()?;

    if var_os("INSIDE_LAMBDA").is_some() {
        lambda!(handler);
    } else {
        for bucket_key in &CONFIG.bucket_keys {
            process_log_file(&CONFIG.bucket_name, &bucket_key)?
        }
    }

    Ok(())
}
