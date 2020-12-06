use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poetry_resolver::resolve;
use std::time::Duration;

fn test_resolve_poetry() {
    resolve("poetry", "1.2.0a0", vec![
        ("poetry-core", ">=1.0.0,<2"),
        ("cleo", ">=0.8.1,<0.9"),
        ("clikit", ">=0.6.2,<0.7"),
        ("crashtest", ">=0.3.0,<0.4"),
        ("requests", ">=2.18,<3"),
        ("cachy", ">=0.3.0,<0.4"),
        ("requests-toolbelt", ">=0.9.1,<0.10"),
        ("pkginfo", ">=1.4,<2"),
        ("html5lib", ">=1.0,<2"),
        ("shellingham", ">=1.1,<2"),
        ("tomlkit", ">=0.7.0,<1.0.0"),
        ("pexpect", ">=4.7.0,<5"),
        ("packaging", ">=20.4,<21"),
        ("virtualenv", ">20.0.26,<21"),
        ("keyring", ">=21.2.0,<22"),
        // missing extra "filecache"
        ("cachecontrol", ">=0.12.4,<0.13"),
    ], vec![
        ("pytest", ">=5.4.3,<6"),
        ("pre-commit", ">=2.6,<3"),
        ("pytest-cov", ">=2.5,<3"),
        ("pytest-mock", ">=1.9,<2"),
        ("tox", ">=3.0,<4"),
        ("pytest-sugar", ">=0.9.2,<0.10"),
        ("httpretty", ">=1.0,<2"),
        ("urllib3", "==1.25.10"),
        ("setuptools-rust", ">=0.11.5,<0.12")
    ]);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolving examples");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(30));
    group.measurement_time(Duration::from_secs(240));
    group.bench_function("poetry", |b| b.iter(test_resolve_poetry));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);