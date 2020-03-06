use std::io::Cursor;

use criterion::{criterion_group, criterion_main, Criterion};

use elb_logs_to_cloudwatch::output::{OutputType, VoidOutput};
use elb_logs_to_cloudwatch::{compile_pipelines, process_log, Pipeline, Pipelines};

const GOOD_LOGS: &str = include_str!("../tests/fixtures/logs.txt");

fn criterion_benchmark(c: &mut Criterion) {
    let raw_pipelines = Pipelines::new(vec![Pipeline {
        filter: "elb_status_code == 200 && user_agent matches \"(Android|axios)\"".to_string(),
        output: OutputType::Void(VoidOutput),
    }]);
    let pipelines = compile_pipelines(&raw_pipelines);

    c.bench_function("compile pipelines", |b| {
        b.iter(|| compile_pipelines(&raw_pipelines))
    });
    c.bench_function("10", |b| {
        b.iter(|| process_log(Cursor::new(GOOD_LOGS), &pipelines).unwrap())
    });
    c.bench_function("100", |b| {
        let monster_log: Vec<&str> = (0..10).map(|_| GOOD_LOGS).collect();
        let monster_log = monster_log.join("");
        b.iter(|| process_log(Cursor::new(monster_log.as_str()), &pipelines).unwrap())
    });
    c.bench_function("1000", |b| {
        let monster_log: Vec<&str> = (0..100).map(|_| GOOD_LOGS).collect();
        let monster_log = monster_log.join("");
        b.iter(|| process_log(Cursor::new(monster_log.as_str()), &pipelines).unwrap())
    });
    c.bench_function("10000", |b| {
        let monster_log: Vec<&str> = (0..1000).map(|_| GOOD_LOGS).collect();
        let monster_log = monster_log.join("");
        b.iter(|| process_log(Cursor::new(monster_log.as_str()), &pipelines).unwrap())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
