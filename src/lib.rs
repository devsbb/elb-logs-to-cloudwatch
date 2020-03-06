pub mod error;
pub mod log_processing;
pub mod output;
pub mod pipelines;
pub mod types;

mod config;
mod handlers;
mod s3;

pub use crate::log_processing::process_log;
pub use crate::pipelines::{compile_pipelines, Pipeline, Pipelines};
