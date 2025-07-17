use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fontgrep::cli;
use clap::Parser;
use std::path::PathBuf;

fn bench_name_ofl(c: &mut Criterion) {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        return;
    }

    c.bench_function("name_ofl", |b| {
        b.iter(|| {
            let args = cli::Cli::parse_from([
                "fontgrep",
                "-n", 
                "OFL",
                black_box(testdata_path.to_str().unwrap())
            ]);
            let _ = cli::execute(args);
        });
    });
}

fn bench_variable_fonts(c: &mut Criterion) {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        return;
    }

    c.bench_function("variable_fonts", |b| {
        b.iter(|| {
            let args = cli::Cli::parse_from([
                "fontgrep",
                "--variable",
                black_box(testdata_path.to_str().unwrap())
            ]);
            let _ = cli::execute(args);
        });
    });
}

fn bench_feature_search(c: &mut Criterion) {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        return;
    }

    c.bench_function("feature_search", |b| {
        b.iter(|| {
            let args = cli::Cli::parse_from([
                "fontgrep",
                "-f",
                "liga,kern",
                black_box(testdata_path.to_str().unwrap())
            ]);
            let _ = cli::execute(args);
        });
    });
}

criterion_group!(benches, bench_name_ofl, bench_variable_fonts, bench_feature_search);
criterion_main!(benches);
