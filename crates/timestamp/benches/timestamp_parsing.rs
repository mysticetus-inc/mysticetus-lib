use criterion::{Criterion, criterion_group, criterion_main};

pub fn parse_timestamps(_c: &mut Criterion) {
    todo!()
}

criterion_group!(parse_benchmarks, parse_timestamps);
criterion_main!(parse_benchmarks);
