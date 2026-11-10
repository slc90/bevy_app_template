pub mod artifact;

use criterion::Criterion;

pub fn run_criterion(name: &str, mut operation: impl FnMut()) {
    let mut criterion = Criterion::default();
    criterion.bench_function(name, |bencher| bencher.iter(&mut operation));
    criterion.final_summary();
}
